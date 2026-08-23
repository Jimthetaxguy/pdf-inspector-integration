# Third-party dependencies

This project is dual-licensed under **MIT OR Apache-2.0**. Every transitive
dependency is permissively licensed and compatible with that choice.

## Direct upstream

### pdf-inspector

- **Source:** https://github.com/firecrawl/pdf-inspector
- **Pinned SHA:** `2f23f07f6e38fd341361554c114d1abe36349ce7`
- **Pinned date:** 2026-04-15
- **License:** MIT
- **Transitive core dep:** `lopdf` @ J-F-Liu/lopdf SHA `7a05512d` (pulled by pdf-inspector, MIT)

### Upgrade checklist

1. Inspect the official upstream tag or commit without editing this workspace
2. Record and verify the new immutable revision
3. Edit workspace `Cargo.toml` → `rev = "<new-sha>"`
4. `cargo build --release` in this workspace
5. Re-run regression corpus (see [`docs/assessment.md`](docs/assessment.md) §5)
6. If green: `cargo install --path crates/pdf-inspector-mcp --force`
7. Update this file with new SHA + date

### Rollback

Revert the `rev` line in `Cargo.toml` and `cargo install` again.

## Full dependency license audit

Generated with `cargo license` on 2026-08-23. The 193-package workspace graph
(191 external packages plus 2 workspace packages) resolves to:

| License set | Crate count | Notes |
|---|---:|---|
| `Apache-2.0 OR MIT` | 136 | Bulk of the Rust ecosystem |
| `MIT` | 29 | Includes `lopdf` and `pdf-inspector` |
| `Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT` | 12 | wasm/wit toolchain crates |
| `MIT OR Unlicense` | 6 | Permissive dual-license choice |
| `Apache-2.0 OR MIT OR Zlib` | 3 | Permissive multi-license choice |
| `Apache-2.0` | 2 | `rmcp` and `rmcp-macros` |
| `(Apache-2.0 OR MIT) AND BSD-3-Clause` | 1 | Compatible conjunctive terms |
| `(Apache-2.0 OR MIT) AND Unicode-3.0` | 1 | Compatible conjunctive terms |
| `0BSD OR Apache-2.0 OR MIT` | 1 | Permissive multi-license choice |
| `Apache-2.0 OR LGPL-2.1-or-later OR MIT` | 1 | `r-efi`; this project selects MIT |
| `Zlib` | 1 | Permissive license |

**Result:** no resolved package requires GPL, AGPL, LGPL, SSPL, BUSL, or
proprietary licensing. The graph passes the repository's cargo-deny license
policy; `r-efi` offers LGPL-or-later or MIT, and this project selects MIT.

To re-run the audit:

```bash
cargo install cargo-license
cargo license --json
```
