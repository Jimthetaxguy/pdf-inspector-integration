# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Rust workspace with `pdf-inspector-skillkit` library and `pdf-inspector-mcp` server binary
- 9 MCP tools: `classify_pdf`, `pdf_to_markdown`, `analyze_layout`, `extract_text_regions`, `extract_table_regions`, `batch_classify`, `identify_tax_form`, `parse_irc_sections`, `split_sec_filing`
- 4 Sweet tax-review demo tools: `list_tax_packages`, `review_tax_package`, `compare_line_items`, `render_review_memo` — deterministic package review, line-item comparison, and Markdown memo rendering over built-in demo packages (1040, 1120, 1065, 1120-S, K-1, 1099 workflows). Bringing the total to 13 MCP tools.
- `CONTEXT.md` project glossary documenting domain vocabulary and module map
- Validation runner example (`cargo run --example validate_domain`)
- Domain modules for tax form identification, IRC section parsing, SEC filing splitting
- `OnceLock<Regex>` cache for all 32 regexes (compile once, reuse)
- `tracing` to stderr (stdout reserved for JSON-RPC), filename-only path logging
- 30s `tokio::time::timeout` per tool handler
- Crates.io metadata (`description`, `license`, `repository`, `keywords`, `categories`)
- Dual MIT / Apache-2.0 licensing
- GitHub Actions CI: fmt + clippy `-D warnings` + test + release build
- Dependabot weekly cargo + actions updates
- README, CHANGELOG, CONTRIBUTING, THIRD_PARTY license audit

### Known limitations
- `parse_irc_sections`: regex captures only the leading section integer, drops decimal/parens (does not handle Treas Reg format) — flagged experimental
- `identify_tax_form`: bank-direct 1099-INTs that render as numeric tables only return `Unknown` (no header text in markdown)
- OCR fallback for scanned PDFs not yet implemented — first scanned PDF returns empty markdown
