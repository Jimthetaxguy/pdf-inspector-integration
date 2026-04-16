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

1. `cd ~/code/third_party/pdf-inspector && git pull --ff-only`
2. Record new HEAD SHA
3. Edit workspace `Cargo.toml` → `rev = "<new-sha>"`
4. `cargo build --release` in this workspace
5. Re-run regression corpus (see `assessment.md` §5)
6. If green: `cargo install --path crates/pdf-inspector-mcp --force`
7. Update this file with new SHA + date

### Rollback

Revert the `rev` line in `Cargo.toml` and `cargo install` again.

## Full dependency license audit

Generated with `cargo license`. Every crate is permissively licensed:

| License set                                            | Crate count | Notes                                                       |
|--------------------------------------------------------|-------------|-------------------------------------------------------------|
| `Apache-2.0 OR MIT`                                    | 138         | Bulk of the Rust ecosystem (serde, tokio, regex, anyhow…)   |
| `Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT`  | 12          | wasm/wit toolchain crates                                   |
| `Apache-2.0 OR MIT OR Zlib`                            | 3           | `miniz_oxide`, `tinyvec`, `tinyvec_macros`                  |
| `0BSD OR Apache-2.0 OR MIT`                            | 1           | `adler2`                                                    |
| `Apache-2.0 OR LGPL-2.1-or-later OR MIT`               | 1           | `r-efi` (we select MIT)                                     |
| `MIT`                                                  | n/a         | `bytes`, `lopdf`, `pdf-inspector`, etc.                     |
| `MIT OR Unlicense`                                     | n/a         | `aho-corasick`, `memchr`, etc.                              |
| `Apache-2.0` (only)                                    | 2           | `rmcp`, `rmcp-macros` — covered by our Apache-2.0 option    |
| `MPL-2.0`                                              | 1           | `option-ext` — file-level copyleft, no taint to our code    |
| `Zlib`                                                 | 1           | `foldhash` — permissive                                     |

**Result:** zero GPL/AGPL/LGPL/SSPL/BUSL/proprietary. Safe for public,
permissive distribution under MIT or Apache-2.0.

To re-run the audit:

```bash
cargo install cargo-license
cargo license
```
