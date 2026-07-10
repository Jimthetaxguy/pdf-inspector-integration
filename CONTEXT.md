# pdf-inspector-integration — project context glossary

**Role:** first-party-active Rust workspace + MCP server for offline PDF intel.  
**Path:** `/Users/jamespustorino/code/pdf-inspector-integration`  
**Remote:** `https://github.com/Jimthetaxguy/pdf-inspector-integration.git`  
**Branch:** `main`

## Purpose

Expose [firecrawl/pdf-inspector](https://github.com/firecrawl/pdf-inspector) over **MCP** so coding agents classify/extract PDFs in milliseconds **without OCR-first**, then layer tax / IRC / SEC / Sweet tax-review helpers as separate tools.

## Domain vocabulary

| Term | Meaning |
|------|---------|
| **classify_pdf** | TextBased / Scanned / Mixed + confidence (~1–10 ms) |
| **pdf_to_markdown** | Born-digital PDF → clean Markdown (headings/tables/lists) |
| **analyze_layout** | Tables, columns, complexity metrics |
| **extract_text_regions / extract_table_regions** | Geometry-bounded extraction (`[x1,y1,x2,y2]`) |
| **batch_classify** | Multi-PDF classify loop |
| **identify_tax_form** | W-2 / 1099 / K-1 / 1040 / 1065 / 1120 / schedules detector |
| **parse_irc_sections** | Title 26 IRC section parser (experimental capture format) |
| **split_sec_filing** | 10-K / 10-Q Item-number splitter |
| **Sweet demo package** | Bundled structured tax-review package (list / review / compare / memo tools) |
| **skillkit** | Library crate with domain modules (`tax`, `irc`, `sec`, `sweet`) |
| **mcp crate** | `pdf-inspector-mcp` binary exposing tools via rmcp |

## Module map

| Path | Role |
|------|------|
| `crates/pdf-inspector-skillkit/` | Domain + extraction helpers (library) |
| `crates/pdf-inspector-skillkit/src/domain/` | `tax.rs`, `irc`, `sec`, `sweet` |
| `crates/pdf-inspector-mcp/` | MCP server binary (`main.rs` tool registration) |
| `docs/` | Handoff, Sweet demo notes |
| `test-corpus/` | Fixtures for validation |
| `scripts/` | Local helpers |

## Real systems

- PDF parsing via git-pinned `pdf-inspector` (firecrawl) + `lopdf` — **offline**, no cloud OCR default
- MCP over stdio for Claude/Codex/Cursor/etc.
- Demo Sweet packages are **synthetic structured examples**, not live client filings

## Verify

```bash
cargo check --workspace
cargo test --workspace
# install binary: cargo install --path crates/pdf-inspector-mcp
```

CI: GitHub Actions badge on README.

## Known limitations (from README)

- Alpha; not all 13 tools live-smoked against real PDFs
- Bank-direct 1099-INTs often `Unknown` for form id
- IRC section-number capture format incomplete
- Sweet tools are demo/synthetic until real packages wired

## Non-goals

- Not a general OCR pipeline (use OCR only when classify says Scanned/Mixed and user opts in)
- Do not commit real client tax PDFs or PII filings into the repo
- Vendor `pdf-inspector` upstream: reference via git rev; do not push upstream

## Cleanup note (2026-07-10)

CONTEXT added. Sweet tax-review domain + MCP tools may land in the same hygiene checkpoint as this glossary.
