//! MCP server for pdf-inspector — exposes classify, extract, and layout
//! tools to coding agents over stdio transport.
//!
//! Tools: classify_pdf, pdf_to_markdown, analyze_layout, batch_classify,
//! extract_text_regions, extract_table_regions, identify_tax_form,
//! split_sec_filing, parse_irc_sections.
//!
//! All tool handlers are wrapped in a 30-second timeout to bound worst-case
//! latency on pathological PDFs. Logs go to stderr — stdout is reserved for
//! the JSON-RPC channel and contaminating it would break the MCP protocol.

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// Per-tool wall-clock cap. Pathological PDFs can spin pdf-inspector for
/// minutes; bound it so the agent caller can recover.
const TOOL_TIMEOUT: Duration = Duration::from_secs(30);

/// Render a uniform JSON error envelope. Matches the shape used by the
/// per-tool error branches so callers see one schema regardless of source.
fn json_error(msg: impl std::fmt::Display) -> String {
    serde_json::json!({ "error": msg.to_string() }).to_string()
}

/// Strip the path down to its file name for logging — never log absolute
/// paths (PII risk: home directory, project layout, customer file names
/// in some flows).
fn log_name(path: &str) -> std::ffi::OsString {
    Path::new(path)
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("<unknown>"))
}

/// Run a tool body with the global 30s timeout. On timeout, return a
/// structured JSON error string (not a panic) so the rmcp tool schema —
/// which expects `String` — stays intact.
async fn with_timeout<F>(tool: &'static str, fut: F) -> String
where
    F: std::future::Future<Output = String>,
{
    match tokio::time::timeout(TOOL_TIMEOUT, fut).await {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!(tool, "tool timed out after 30s");
            json_error(format!("tool '{tool}' timed out after 30s"))
        }
    }
}

/// Input for single-path tools (classify, markdown, analyze).
#[derive(Deserialize, JsonSchema)]
struct PathInput {
    /// Absolute or relative path to the PDF file.
    path: String,
}

/// Input for batch_classify tool.
#[derive(Deserialize, JsonSchema)]
struct BatchClassifyInput {
    /// List of absolute or relative paths to PDF files.
    paths: Vec<String>,
}

/// A single region on a page specified in PDF points with top-left origin.
#[derive(Deserialize, JsonSchema)]
struct RegionSpec {
    /// 0-indexed page number.
    page: u32,
    /// List of rectangles `[x1, y1, x2, y2]` in PDF points (top-left origin).
    rects: Vec<[f32; 4]>,
}

/// Input for extract_text_regions and extract_table_regions tools.
#[derive(Deserialize, JsonSchema)]
struct RegionInput {
    /// Absolute or relative path to the PDF file.
    path: String,
    /// Regions to extract from, specified as (page, rects) pairs.
    regions: Vec<RegionSpec>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PdfInspectorServer {
    tool_router: ToolRouter<Self>,
}

impl PdfInspectorServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl PdfInspectorServer {
    /// Classify a PDF as TextBased, Scanned, ImageBased, or Mixed.
    #[tool(
        description = "Classify a PDF as TextBased/Scanned/ImageBased/Mixed with confidence score and per-page OCR hints"
    )]
    async fn classify_pdf(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "classify_pdf", path = ?log_name(&path), "tool invoked");
        with_timeout("classify_pdf", async move {
            match pdf_inspector_skillkit::classify(&path) {
                Ok(info) => serde_json::to_string_pretty(&info).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Convert a PDF to clean Markdown.
    #[tool(
        description = "Convert a PDF to clean Markdown with headings, tables, lists, and code blocks"
    )]
    async fn pdf_to_markdown(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "pdf_to_markdown", path = ?log_name(&path), "tool invoked");
        with_timeout("pdf_to_markdown", async move {
            match pdf_inspector_skillkit::process(&path) {
                Ok(info) => serde_json::to_string_pretty(&info).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Analyze layout complexity of a PDF (tables, multi-column, etc.).
    #[tool(
        description = "Analyze layout complexity of a PDF — returns tables detected, multi-column indicators, and other layout metrics"
    )]
    async fn analyze_layout(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "analyze_layout", path = ?log_name(&path), "tool invoked");
        with_timeout("analyze_layout", async move {
            match pdf_inspector_skillkit::analyze(&path) {
                Ok(info) => serde_json::to_string_pretty(&info).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Batch classify multiple PDFs sequentially.
    #[tool(
        description = "Classify multiple PDFs — returns array of {path, classification} objects"
    )]
    async fn batch_classify(&self, params: Parameters<BatchClassifyInput>) -> String {
        let paths = params.0.paths;
        tracing::debug!(tool = "batch_classify", count = paths.len(), "tool invoked");
        with_timeout("batch_classify", async move {
            let results: Vec<serde_json::Value> = paths
                .into_iter()
                .map(|path| match pdf_inspector_skillkit::classify(&path) {
                    Ok(info) => serde_json::json!({
                        "path": path,
                        "classification": info
                    }),
                    Err(e) => {
                        tracing::warn!(error = %e, "tool failed");
                        serde_json::json!({
                            "path": path,
                            "error": e.to_string()
                        })
                    }
                })
                .collect();
            serde_json::to_string_pretty(&results).unwrap_or_else(json_error)
        })
        .await
    }

    /// Extract text from specified regions of a PDF.
    ///
    /// Each region is defined by a page number (0-indexed) and a list of
    /// bounding rectangles `[x1, y1, x2, y2]` in PDF points with top-left origin.
    #[tool(
        description = "Extract text from specified rectangular regions of a PDF — returns text per region with OCR hints"
    )]
    async fn extract_text_regions(&self, params: Parameters<RegionInput>) -> String {
        let path = params.0.path;
        let regions = params.0.regions;
        tracing::debug!(tool = "extract_text_regions", path = ?log_name(&path), "tool invoked");
        with_timeout("extract_text_regions", async move {
            let page_regions: Vec<(u32, Vec<[f32; 4]>)> =
                regions.into_iter().map(|r| (r.page, r.rects)).collect();
            match pdf_inspector_skillkit::extract_text_regions(&path, &page_regions) {
                Ok(results) => serde_json::to_string_pretty(&results).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Extract tables from specified regions of a PDF as markdown pipe-tables.
    ///
    /// Similar to extract_text_regions but runs table detection and returns
    /// markdown pipe-tables instead of flat text.
    #[tool(
        description = "Extract tables from specified rectangular regions of a PDF as markdown pipe-tables"
    )]
    async fn extract_table_regions(&self, params: Parameters<RegionInput>) -> String {
        let path = params.0.path;
        let regions = params.0.regions;
        tracing::debug!(tool = "extract_table_regions", path = ?log_name(&path), "tool invoked");
        with_timeout("extract_table_regions", async move {
            let page_regions: Vec<(u32, Vec<[f32; 4]>)> =
                regions.into_iter().map(|r| (r.page, r.rects)).collect();
            match pdf_inspector_skillkit::extract_table_regions(&path, &page_regions) {
                Ok(results) => serde_json::to_string_pretty(&results).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Identify the type of tax form in a PDF (W-2, 1099, K-1, 1040, schedules).
    #[tool(
        description = "Identify the type of tax form in a PDF (W-2, 1099, K-1, 1040, schedules)"
    )]
    async fn identify_tax_form(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "identify_tax_form", path = ?log_name(&path), "tool invoked");
        with_timeout("identify_tax_form", async move {
            match pdf_inspector_skillkit::domain::tax::identify_tax_form(&path) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Split a SEC 10-K/10-Q filing into sections by Item number.
    #[tool(
        description = "Split a SEC 10-K/10-Q filing into sections by Item number — returns array of {name, item_number, content, char_offset}"
    )]
    async fn split_sec_filing(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "split_sec_filing", path = ?log_name(&path), "tool invoked");
        with_timeout("split_sec_filing", async move {
            match pdf_inspector_skillkit::domain::sec::split_sec_filing(&path) {
                Ok(sections) => serde_json::to_string_pretty(&sections).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }

    /// Parse IRC (Internal Revenue Code) sections from a Title 26 PDF.
    #[tool(
        description = "Parse IRC (Internal Revenue Code) sections from a Title 26 PDF — returns structured sections with §numbers, titles, and subsections"
    )]
    async fn parse_irc_sections(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        tracing::debug!(tool = "parse_irc_sections", path = ?log_name(&path), "tool invoked");
        with_timeout("parse_irc_sections", async move {
            match pdf_inspector_skillkit::domain::irc::parse_irc_sections(&path) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_error),
                Err(e) => {
                    tracing::warn!(error = %e, "tool failed");
                    json_error(e)
                }
            }
        })
        .await
    }
}

#[tool_handler]
impl ServerHandler for PdfInspectorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "PDF classification, text extraction, and layout analysis. \
             Fast (~10ms classify, ~200ms extract), offline, no OCR.",
            )
            .with_server_info(rmcp::model::Implementation::new(
                "pdf-inspector-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CRITICAL: stdout is the MCP JSON-RPC channel. All logs MUST go to stderr
    // or the protocol breaks. RUST_LOG can override; default is `info`.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("pdf-inspector-mcp starting");

    let transport = rmcp::transport::io::stdio();
    let server = PdfInspectorServer::new();
    let service = server.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}
