# Sweet Tax Review Demo

This repo now includes a thin, deterministic Sweet review layer on top of the
existing PDF inspection primitives.

The goal is not to claim production tax-review coverage. The goal is to make the
Harris pilot architecture concrete:

1. Use `identify_tax_form`, markdown extraction, and region extraction to turn
   Harris PDF exports into structured facts.
2. Feed those facts into deterministic review checks.
3. Return structured findings and a reviewer-ready memo through MCP.

## Built-In Demo Packages

Run:

```bash
cargo run -p pdf-inspector-skillkit --example sweet_review_demo
```

Available package ids:

| Package | Coverage |
|---|---|
| `demo_1040_w2_schedule_c` | 1040, W-2, Schedule C, 1099-INT |
| `demo_1120_c_corp` | C corporation depreciation tie-out |
| `demo_1065_partnership` | Partnership Schedule K and K-1 allocations |
| `demo_1120s_s_corp` | S corporation shareholder K-1 checks |
| `demo_k1_partner` | Recipient-side K-1 review |
| `demo_1099_bundle` | 1099-INT, 1099-DIV, and 1099-NEC source matching |

Run a specific package:

```bash
cargo run -p pdf-inspector-skillkit --example sweet_review_demo -- demo_1099_bundle
```

## MCP Calls

The server exposes four Sweet-facing tools:

```text
list_tax_packages
review_tax_package
compare_line_items
render_review_memo
```

The shared tax-form detector now includes explicit entity-return variants for
Form 1065, Form 1120, and Form 1120-S so Sweet package reviews do not need to
route those demos through a generic `unknown` form type.

After installing `pdf-inspector-mcp`, call a structured review:

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sweet-demo","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"review_tax_package","arguments":{"package_id":"demo_1040_w2_schedule_c"}}}' \
  | pdf-inspector-mcp
```

Compare one source value against one return value:

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sweet-demo","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"compare_line_items","arguments":{"label":"W-2 wages","return_reference":"Form 1040 line 1a","source_reference":"W-2 Box 1","return_amount":84732,"source_amount":87432,"tolerance":0}}}' \
  | pdf-inspector-mcp
```

Render a Markdown memo:

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sweet-demo","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"render_review_memo","arguments":{"package_id":"demo_1099_bundle"}}}' \
  | pdf-inspector-mcp
```

## Boundary

The Sweet package tools currently use synthetic structured facts. They are the
review layer. The extraction layer is already present in the existing tools:

- `identify_tax_form`
- `pdf_to_markdown`
- `extract_text_regions`
- `extract_table_regions`

The Harris-specific next step is to replace synthetic values with facts parsed
from sanitized Harris exports, then lock each parser path with fixtures and
reviewer-confirmed expected findings.
