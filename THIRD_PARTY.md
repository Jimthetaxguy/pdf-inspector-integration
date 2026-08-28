# Third-party dependencies

This project is dual-licensed under **MIT OR Apache-2.0**. Every transitive
dependency is permissively licensed and compatible with that choice.

## Direct upstream

### pdf-inspector

- **Source:** https://github.com/firecrawl/pdf-inspector
- **Package:** crates.io `pdf-inspector 1.17.0`
- **Registry package:** crates.io `1.17.0`, checksum `6cdfc6057e1b38a2ae84490c5e64abc5c81738d4d5ac1ccc55cf1a2c9b87334e`
- **Upstream Git context:** `main` @ `23cf1ad7b37eec6e3a21df61f8e6d5dce66c46bd`; latest visible tag `v1.15.0` @ `06a9bab6b3309309503f2db17851389cee094a62`
- **License:** MIT
- **Transitive core dep:** `lopdf 0.42.0` from crates.io (MIT)

### anydoc

- **Source:** https://github.com/firecrawl/anydoc
- **Package:** crates.io `anydoc 0.2.4`
- **Upstream release:** `v0.2.4` @ `42bf1c5ecdde9eb0d96d6bd75a9e6698cf93b14c`
- **License:** MIT
- **Integration status:** Resolved in the workspace for parser convergence; bounded DOCX, exact `.pptx`, exact `.xlsx`, exact `.ods`, exact `.odt`, exact `.odp`, and strict EPUB runtime paths are exposed through the local provider-neutral contract. Macro-enabled, binary, legacy, hidden, externally linked, encrypted, active, and incomplete spreadsheet variants remain disabled; ODT is limited to visible, well-formed, exact-mimetype text packages; EPUB is limited to strict EPUB 3 packages with complete, local, inactive spine content.

### Upgrade checklist

1. Inspect the official upstream tag or commit without editing this workspace
2. Record and verify the new immutable revision or registry checksum
3. Update workspace `Cargo.toml` and `Cargo.lock` deliberately
4. Run the locked build, regression corpus, and security gates
5. Update this file and `docs/upstream-provenance.md` with the new evidence

### Rollback

Revert the dependency and lockfile changes, then rerun the locked verification gates.

## Full dependency license audit

Generated with `cargo license --json` on 2026-08-28. The 211-package workspace graph
(209 external packages plus 2 workspace packages) resolves to:

| License set | Crate count | Notes |
|---|---:|---|
| `Apache-2.0 OR MIT` | 141 | Bulk of the Rust ecosystem |
| `MIT` | 35 | Includes `anydoc`, `lopdf`, and `pdf-inspector` |
| `Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT` | 14 | wasm/wit toolchain crates |
| `MIT OR Unlicense` | 8 | Permissive dual-license choice |
| `Apache-2.0 OR MIT OR Zlib` | 3 | Permissive multi-license choice |
| `Apache-2.0` | 3 | Includes `rmcp` and `rmcp-macros` |
| `(Apache-2.0 OR MIT) AND BSD-3-Clause` | 1 | Compatible conjunctive terms |
| `(Apache-2.0 OR MIT) AND Unicode-3.0` | 1 | Compatible conjunctive terms |
| `0BSD OR Apache-2.0 OR MIT` | 1 | Permissive multi-license choice |
| `Apache-2.0 OR BSL-1.0` | 1 | `ryu`; this project selects Apache-2.0 |
| `Apache-2.0 OR LGPL-2.1-or-later OR MIT` | 1 | `r-efi`; this project selects MIT |
| `Zlib` | 2 | Permissive license |

**Result:** no resolved package requires GPL, AGPL, LGPL, SSPL, BUSL, or
proprietary licensing. The graph passes the repository's cargo-deny license
policy; `r-efi` offers LGPL-or-later or MIT, and this project selects MIT; `ryu`
offers Apache-2.0 or BSL-1.0, and this project selects Apache-2.0.

To re-run the audit:

```bash
cargo install cargo-license
cargo license --json
```
