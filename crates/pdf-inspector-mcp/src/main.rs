//! MCP server for pdf-inspector — exposes classify, extract, and layout
//! tools to coding agents over stdio transport.
//!
//! Tools: classify_pdf, pdf_to_markdown, analyze_layout, batch_classify,
//! extract_text_regions, extract_table_regions, identify_tax_form,
//! split_sec_filing, parse_irc_sections, list_tax_packages,
//! review_tax_package, compare_line_items, render_review_memo.
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
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
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

/// Run a future with a wall-clock timeout. On timeout, return a structured
/// JSON error string (not a panic) so the rmcp tool schema — which expects
/// `String` — stays intact.
///
/// `timeout` is a parameter (rather than always reading the `TOOL_TIMEOUT`
/// constant) so this is unit-testable with a short duration instead of
/// waiting out the real 30s production timeout.
///
/// For this to actually preempt a caller, `fut` must contain a genuine
/// await point. `tokio::time::timeout` races `fut` against a timer inside
/// a single task's poll loop — if `fut` never yields (e.g. it runs a
/// synchronous, CPU-bound closure inline with no `.await`), the executor
/// can't interleave the timer, so the timeout can never fire until the
/// work finishes on its own. `dispatch` below avoids this by running
/// blocking work via `tokio::task::spawn_blocking`, whose `JoinHandle`
/// await is a real yield point.
async fn with_timeout<F>(tool: &'static str, timeout: Duration, fut: F) -> String
where
    F: std::future::Future<Output = String>,
{
    match tokio::time::timeout(timeout, fut).await {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!(tool, ?timeout, "tool timed out");
            json_error(format!("tool '{tool}' timed out after {timeout:?}"))
        }
    }
}

/// Map a `JoinError` from a `spawn_blocking` task (panic or cancellation)
/// into the same structured JSON error envelope used elsewhere, so callers
/// always see `{"error": "..."}` regardless of failure mode.
fn join_error_to_json(tool: &'static str, err: tokio::task::JoinError) -> String {
    let reason = if err.is_panic() {
        "panicked".to_string()
    } else if err.is_cancelled() {
        "was cancelled".to_string()
    } else {
        err.to_string()
    };
    tracing::warn!(tool, error = %reason, "blocking task failed");
    json_error(format!("tool '{tool}' failed: blocking task {reason}"))
}

/// Common dispatch envelope: log invocation, run the (CPU-bound,
/// synchronous) work on tokio's blocking thread pool under the timeout,
/// serialize success or render error. Captures the boilerplate shared by
/// 12 of the 13 tool handlers (batch_classify is bespoke — it folds
/// per-item errors into the response array rather than failing the whole
/// call).
///
/// `work` runs inside `tokio::task::spawn_blocking` rather than inline: it
/// is CPU-bound and can run for a while on pathological PDFs, and running
/// it directly on the async executor thread both blocks that worker for
/// other tasks and — since it has no `.await` inside — starves the
/// `with_timeout` timer of any point at which it could fire.
async fn dispatch<T, E, F>(tool: &'static str, target: impl AsRef<OsStr>, work: F) -> String
where
    T: Serialize + Send + 'static,
    E: std::fmt::Display + Send + 'static,
    F: FnOnce() -> Result<T, E> + Send + 'static,
{
    tracing::debug!(tool, target = ?target.as_ref(), "tool invoked");
    with_timeout(tool, TOOL_TIMEOUT, async move {
        match tokio::task::spawn_blocking(work).await {
            Ok(Ok(v)) => serde_json::to_string_pretty(&v).unwrap_or_else(json_error),
            Ok(Err(e)) => {
                tracing::warn!(tool, error = %e, "tool failed");
                json_error(e)
            }
            Err(join_err) => join_error_to_json(tool, join_err),
        }
    })
    .await
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

/// Input for Sweet package review and memo tools.
#[derive(Deserialize, JsonSchema)]
struct SweetPackageInput {
    /// Demo package id, such as `demo_1040_w2_schedule_c`.
    package_id: String,
}

/// Input for comparing one return line against one source document line.
#[derive(Deserialize, JsonSchema)]
struct SweetCompareLineItemsInput {
    /// Human-readable label for the comparison.
    label: String,
    /// Return form and line reference, such as `Form 1040 line 1a`.
    return_reference: String,
    /// Source document reference, such as `W-2 Box 1`.
    source_reference: String,
    /// Amount shown on the return. Demo values are whole dollars.
    return_amount: i64,
    /// Amount shown on the source document. Demo values are whole dollars.
    source_amount: i64,
    /// Allowed absolute difference before the comparison is flagged.
    #[serde(default)]
    tolerance: Option<i64>,
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
        let target = log_name(&path);
        dispatch("classify_pdf", target, move || {
            pdf_inspector_skillkit::classify(&path)
        })
        .await
    }

    /// Convert a PDF to clean Markdown.
    #[tool(
        description = "Convert a PDF to clean Markdown with headings, tables, lists, and code blocks"
    )]
    async fn pdf_to_markdown(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        let target = log_name(&path);
        dispatch("pdf_to_markdown", target, move || {
            pdf_inspector_skillkit::process(&path)
        })
        .await
    }

    /// Analyze layout complexity of a PDF (tables, multi-column, etc.).
    #[tool(
        description = "Analyze layout complexity of a PDF — returns tables detected, multi-column indicators, and other layout metrics"
    )]
    async fn analyze_layout(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        let target = log_name(&path);
        dispatch("analyze_layout", target, move || {
            pdf_inspector_skillkit::analyze(&path)
        })
        .await
    }

    /// Batch classify multiple PDFs sequentially.
    ///
    /// Bespoke because per-item errors are folded into the response array
    /// rather than failing the whole call — so the dispatch helper doesn't fit.
    #[tool(
        description = "Classify multiple PDFs — returns array of {path, classification} objects"
    )]
    async fn batch_classify(&self, params: Parameters<BatchClassifyInput>) -> String {
        let paths = params.0.paths;
        tracing::debug!(tool = "batch_classify", count = paths.len(), "tool invoked");
        with_timeout("batch_classify", TOOL_TIMEOUT, async move {
            let work = move || -> Vec<serde_json::Value> {
                paths
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
                    .collect()
            };
            match tokio::task::spawn_blocking(work).await {
                Ok(results) => serde_json::to_string_pretty(&results).unwrap_or_else(json_error),
                Err(join_err) => join_error_to_json("batch_classify", join_err),
            }
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
        let target = log_name(&path);
        dispatch("extract_text_regions", target, move || {
            let page_regions: Vec<(u32, Vec<[f32; 4]>)> =
                regions.into_iter().map(|r| (r.page, r.rects)).collect();
            pdf_inspector_skillkit::extract_text_regions(&path, &page_regions)
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
        let target = log_name(&path);
        dispatch("extract_table_regions", target, move || {
            let page_regions: Vec<(u32, Vec<[f32; 4]>)> =
                regions.into_iter().map(|r| (r.page, r.rects)).collect();
            pdf_inspector_skillkit::extract_table_regions(&path, &page_regions)
        })
        .await
    }

    /// Identify the type of tax form in a PDF (W-2, 1099, K-1, 1040, schedules).
    #[tool(
        description = "Identify the type of tax form in a PDF (W-2, 1099, K-1, 1040, schedules)"
    )]
    async fn identify_tax_form(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        let target = log_name(&path);
        dispatch("identify_tax_form", target, move || {
            pdf_inspector_skillkit::domain::tax::identify_tax_form(&path)
        })
        .await
    }

    /// Split a SEC 10-K/10-Q filing into sections by Item number.
    #[tool(
        description = "Split a SEC 10-K/10-Q filing into sections by Item number — returns array of {name, item_number, content, char_offset}"
    )]
    async fn split_sec_filing(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        let target = log_name(&path);
        dispatch("split_sec_filing", target, move || {
            pdf_inspector_skillkit::domain::sec::split_sec_filing(&path)
        })
        .await
    }

    /// Parse IRC (Internal Revenue Code) sections from a Title 26 PDF.
    #[tool(
        description = "Parse IRC (Internal Revenue Code) sections from a Title 26 PDF — returns structured sections with §numbers, titles, and subsections"
    )]
    async fn parse_irc_sections(&self, params: Parameters<PathInput>) -> String {
        let path = params.0.path;
        let target = log_name(&path);
        dispatch("parse_irc_sections", target, move || {
            pdf_inspector_skillkit::domain::irc::parse_irc_sections(&path)
        })
        .await
    }

    /// List built-in Sweet tax review demo packages.
    #[tool(
        description = "List built-in Sweet tax review demo packages across 1040, 1120, 1065, 1120-S, K-1, and 1099 workflows"
    )]
    async fn list_tax_packages(&self) -> String {
        dispatch("list_tax_packages", "sweet-demo-packages", || {
            Ok::<_, std::convert::Infallible>(
                pdf_inspector_skillkit::domain::sweet::list_tax_packages(),
            )
        })
        .await
    }

    /// Review a built-in Sweet tax package and return structured findings.
    #[tool(
        description = "Run deterministic Sweet tax review checks for a built-in demo package and return structured findings"
    )]
    async fn review_tax_package(&self, params: Parameters<SweetPackageInput>) -> String {
        let package_id = params.0.package_id;
        let target = package_id.clone();
        dispatch("review_tax_package", target, move || {
            pdf_inspector_skillkit::domain::sweet::review_tax_package(&package_id)
        })
        .await
    }

    /// Compare one return line item against one source document value.
    #[tool(
        description = "Compare one tax return line against one source document line with an optional tolerance"
    )]
    async fn compare_line_items(&self, params: Parameters<SweetCompareLineItemsInput>) -> String {
        let input = params.0;
        let target = input.label.clone();
        dispatch("compare_line_items", target, move || {
            Ok::<_, std::convert::Infallible>(
                pdf_inspector_skillkit::domain::sweet::compare_line_items(
                    pdf_inspector_skillkit::domain::sweet::LineComparisonInput {
                        label: input.label,
                        return_reference: input.return_reference,
                        source_reference: input.source_reference,
                        return_amount: input.return_amount,
                        source_amount: input.source_amount,
                        tolerance: input.tolerance,
                    },
                ),
            )
        })
        .await
    }

    /// Render a Markdown review memo for a built-in Sweet demo package.
    #[tool(description = "Render a Markdown tax review memo for a built-in Sweet demo package")]
    async fn render_review_memo(&self, params: Parameters<SweetPackageInput>) -> String {
        let package_id = params.0.package_id;
        let target = package_id.clone();
        dispatch("render_review_memo", target, move || {
            pdf_inspector_skillkit::domain::sweet::render_review_memo(&package_id)
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
             Fast (~10ms classify, ~200ms extract), offline, no OCR. \
             Includes Sweet tax-review demo tools for deterministic package \
             review, line-item comparison, and Markdown memo rendering.",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// A blocking closure that sleeps far longer than its timeout must
    /// still return the `{"error": "... timed out ..."}` envelope promptly,
    /// rather than hanging until the closure finishes. This is the
    /// regression test for the fix: `with_timeout` races its future
    /// against a timer, and that future must contain a real `.await`
    /// point (here, `spawn_blocking`'s `JoinHandle`) for the timer to
    /// ever get a chance to win.
    #[tokio::test]
    async fn with_timeout_preempts_a_slow_blocking_closure() {
        let start = Instant::now();

        let result = with_timeout("slow_tool", Duration::from_millis(50), async {
            match tokio::task::spawn_blocking(|| {
                std::thread::sleep(Duration::from_millis(200));
                "should not be observed before the timeout fires".to_string()
            })
            .await
            {
                Ok(s) => s,
                Err(e) => join_error_to_json("slow_tool", e),
            }
        })
        .await;

        let elapsed = start.elapsed();

        // The 50ms timeout must win the race against the 200ms blocking
        // closure — proving the timeout actually preempted rather than
        // blocking the executor until `work` finished on its own.
        assert!(
            elapsed < Duration::from_millis(200),
            "with_timeout did not preempt the slow closure: took {elapsed:?}"
        );

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("timeout must return a JSON error envelope");
        let msg = parsed["error"]
            .as_str()
            .expect("envelope must have a string `error` key");
        assert!(
            msg.contains("timed out"),
            "expected a timeout message, got: {msg}"
        );
    }

    /// A closure that completes well within the timeout must return its
    /// own result unaffected — the timeout should not be a false trigger.
    #[tokio::test]
    async fn with_timeout_passes_through_fast_work() {
        let result = with_timeout("fast_tool", Duration::from_millis(200), async {
            match tokio::task::spawn_blocking(|| "ok".to_string()).await {
                Ok(s) => s,
                Err(e) => join_error_to_json("fast_tool", e),
            }
        })
        .await;

        assert_eq!(result, "ok");
    }
}
