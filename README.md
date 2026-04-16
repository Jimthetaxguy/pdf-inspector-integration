# pdf-inspector-integration

[![CI](https://github.com/Jimthetaxguy/pdf-inspector-integration/actions/workflows/ci.yml/badge.svg)](https://github.com/Jimthetaxguy/pdf-inspector-integration/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Rust workspace plus MCP server wrapping the [firecrawl/pdf-inspector](https://github.com/firecrawl/pdf-inspector)
library for offline, fast PDF classification and extraction, with tax-form,
IRC, and SEC-filing domain helpers exposed as separate tools.

## Status

**Alpha.** The 9 MCP tools compile and load; validation against real PDFs is
partial. Use ground-truth fixtures of your own to confirm fitness for any
specific workflow.

Validation snapshot (see [`docs/HANDOFF.md`](docs/HANDOFF.md) for full log):

| Tool | Real-PDF validated? | Notes |
|---|---|---|
| `classify_pdf` | Yes | 98 PDFs, 100% text-based corpus, ~1-10 ms |
| `pdf_to_markdown` | Yes | 4-page doc in ~11 ms |
| `analyze_layout` | Compile-tested only | No live MCP smoke yet |
| `extract_text_regions` | Compile-tested only | API verified, no live MCP smoke |
| `extract_table_regions` | Compile-tested only | API verified, no live MCP smoke |
| `batch_classify` | Compile-tested only | Loops `classify_pdf` |
| `identify_tax_form` | Yes (10/12) | Bank-direct 1099-INTs return Unknown — see Known limitations |
| `parse_irc_sections` | Partial | Counts non-zero on Treas Reg corpus, but section-number capture is wrong format — see CHANGELOG |
| `split_sec_filing` | Yes (26/~26) | Apple 10-K FY2024, all canonical Items |

## Why

Coding agents that touch PDFs almost always reach for OCR first, even when
the source is born-digital. That round-trip costs seconds-to-minutes per
document and discards the structural information already present in the
PDF (text positions, headings, table boundaries).

`pdf-inspector` reads the PDF directly: it classifies (TextBased / Scanned /
Mixed) in single-digit milliseconds and extracts to clean Markdown without
calling out to any OCR engine. This project exposes that capability over the
Model Context Protocol so any MCP-aware agent (Claude Code, Codex, Cursor,
Gemini, OpenCode) can call it identically.

On top of the generic primitives, three domain-specific tools encode patterns
we use ourselves: tax-form identification (W-2, 1099, K-1, 1040), IRC section
parsing for Title 26 PDFs, and SEC 10-K / 10-Q section splitting. These are
layered as separate MCP tools rather than baked into the core extractor so
the upstream surface stays clean.

## The 9 tools

| Tool | What it does | Status |
|---|---|---|
| `classify_pdf` | TextBased / Scanned / Mixed classification with confidence | stable |
| `pdf_to_markdown` | Full PDF to clean Markdown with headings, tables, lists | stable |
| `analyze_layout` | Tables, columns, complexity metrics | beta |
| `extract_text_regions` | Text from `[x1,y1,x2,y2]` rectangles | beta |
| `extract_table_regions` | Tables from rectangles as Markdown pipe-tables | beta |
| `batch_classify` | Classify many PDFs in one call | beta |
| `identify_tax_form` | Detect W-2 / 1099 / K-1 / 1040 / schedules | beta |
| `parse_irc_sections` | Section parser for Title 26 IRC PDFs | experimental |
| `split_sec_filing` | 10-K / 10-Q section splitter by Item number | beta |

## Install

### From source

```bash
git clone https://github.com/Jimthetaxguy/pdf-inspector-integration
cd pdf-inspector-integration
cargo install --path crates/pdf-inspector-mcp
```

This places `pdf-inspector-mcp` in your Cargo bin directory (typically
`~/.cargo/bin/`). To pin a known location, use:

```bash
cargo install --root /usr/local --path crates/pdf-inspector-mcp
```

### Wire into Claude Code

Add an entry to `~/.claude/mcp.json` (or your equivalent MCP-client config):

```json
{
  "mcpServers": {
    "pdf-inspector": {
      "command": "<HOME>/.cargo/bin/pdf-inspector-mcp",
      "args": []
    }
  }
}
```

Replace `<HOME>/.cargo/bin/pdf-inspector-mcp` with the actual absolute path
returned by `which pdf-inspector-mcp` after install. If you used
`cargo install --root /usr/local`, the path is `/usr/local/bin/pdf-inspector-mcp`.

## Quick start

The server speaks JSON-RPC over stdio. A minimal `tools/call` for
`classify_pdf`:

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"classify_pdf","arguments":{"path":"/path/to/file.pdf"}}}' \
  | pdf-inspector-mcp
```

The same pattern works for any of the 9 tools. Two more one-liners (drop in
the `printf` block above, replacing the third line):

```text
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"identify_tax_form","arguments":{"path":"/path/to/W2.pdf"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"split_sec_filing","arguments":{"path":"/path/to/10-K.pdf"}}}
```

The actual response payload is JSON-encoded inside the standard MCP
`{"content":[{"type":"text","text":...}]}` envelope.

## Architecture

A caller (any MCP-aware agent) speaks JSON-RPC over stdio to the
`pdf-inspector-mcp` binary. The binary dispatches to a thin facade crate
(`pdf-inspector-skillkit`) which wraps the upstream `pdf-inspector` library.
Domain modules (tax / sec / irc) are siblings of the core extractor — each
composes the primitives with format-specific knowledge but never modifies
the upstream surface.

```
  caller (Claude Code, Codex, ...)
        |
        | JSON-RPC over stdio (MCP)
        v
  pdf-inspector-mcp        (rmcp 1.4 server)
        |
        v
  pdf-inspector-skillkit   (facade lib)
   |       |
   |       +-- domain::tax     (identify_tax_form)
   |       +-- domain::sec     (split_sec_filing)
   |       +-- domain::irc     (parse_irc_sections)
   v
  pdf-inspector             (upstream, SHA-pinned)
```

## Development

| Task | Command |
|---|---|
| Build | `cargo build --workspace` |
| Test | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Validate domain tool against a PDF | `cargo run --example validate_domain -- <tax\|irc\|sec> <pdf-path>` |

## License

Dual-licensed under either of:

- MIT License — see [LICENSE-MIT](LICENSE-MIT)
- Apache License 2.0 — see [LICENSE-APACHE](LICENSE-APACHE)

at your option.

Every transitive dependency is permissively licensed (MIT, Apache-2.0,
BSD, MPL-2.0 file-level, or Zlib). See [THIRD_PARTY.md](THIRD_PARTY.md)
for the full audit.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code standards, and the PR
checklist (including a hard rule against committing real personal or
financial PDFs as test fixtures).
