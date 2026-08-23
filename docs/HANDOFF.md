# anydoc-enhanced — public handoff

**Last reconciled:** 2026-08-23
**Repository:** <https://github.com/Jimthetaxguy/anydoc-enhanced>
**Status:** PDF MCP baseline exists; AnyDoc integration is planned, not shipped.

This is the public, repository-relative entry point for future work. Do not add
home-directory paths, private corpus locations, credentials, internal agent
configuration, or identifying source-document details.

## Current system

The workspace exposes 13 MCP tools over stdio:

- Six generic PDF tools: classify, Markdown, layout, batch, and two region
  extractors.
- Three domain parsers: tax-form identification, IRC section parsing, and SEC
  filing splitting.
- Four deterministic synthetic tax-review demo tools.

The dependency direction is:

```text
pdf-inspector-mcp
        |
pdf-inspector-skillkit
        |
firecrawl/pdf-inspector
```

`pdf-inspector-skillkit` is the only crate that may call parser libraries.
MCP handlers and domain modules must depend on the skillkit boundary.

## Current source map

| Path | Responsibility |
|---|---|
| `crates/pdf-inspector-skillkit/src/lib.rs` | PDF facade, validation, and serialized result types |
| `crates/pdf-inspector-skillkit/src/domain/` | Tax, IRC, SEC, and synthetic review logic |
| `crates/pdf-inspector-mcp/src/main.rs` | MCP schemas, tool registration, dispatch, and timeout response handling |
| `scripts/check-public-hygiene.sh` | Tracked-text PII/path guard used locally and in CI |
| `test-corpus/README.md` | Public fixture provenance and contributor gate |
| `docs/dependency-pr-review-2026-08-22.md` | Live review of dependency PRs #14–#18 |
| `docs/anydoc-integration-plan.md` | Authoritative dependency-ordered AnyDoc plan |
| `CONTEXT.md` | Stable project vocabulary and boundaries |

## Verified constraints

- Existing text-PDF calls are local and do not require a hosted API.
  Image-only PDFs report OCR requirements, while mixed PDFs can produce
  partial extraction with page-level OCR diagnostics; no OCR engine is
  currently shipped.
- Input paths are canonicalized and capped at 50 MiB.
- MCP handlers return after a 30-second Tokio timeout, but a timed-out
  `spawn_blocking` task is not terminated. The AnyDoc plan therefore requires a
  killable worker boundary before untrusted non-PDF parsing is exposed.
- AnyDoc `v0.2.3` is a native Rust library, MIT licensed, and uses
  `pdf-inspector 1.14.2` for PDFs.
- This workspace still uses an older Git-source `pdf-inspector 0.1.0` revision.
  Parser convergence is the first AnyDoc implementation gate.
- AnyDoc CSV and RTF parsing remain disabled in the proposed first release
  because upstream issue #104 documents memory-exhaustion risk.

## Plan invariant

Existing PDF tool names and schemas must remain backward compatible throughout
the AnyDoc work.

## Public-data boundary

Only synthetic or demonstrably redistributable fixtures may enter Git. The
tracked source PDFs contain public U.S. Code text. Samples 4 and 5 are empty
slots and have no benchmark rows.

Before adding a fixture, follow the manifest and metadata checks in
[`test-corpus/README.md`](../test-corpus/README.md). Never discover arbitrary
files from a contributor's home directories during tests.

## Verification

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo metadata --locked --format-version 1
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo audit
bash scripts/check-public-hygiene.sh
```

The candidate branch configures CI to run:

```bash
cargo deny --all-features --locked check advisories licenses bans sources
```

Treat that policy as active on the public default branch only after this exact
head passes its pull-request checks and is merged.

When MCP schemas change, initialize the built binary over stdio and verify the
exact tool-name set. When parser dependencies change, also assert that
`cargo tree -d` does not show two `pdf-inspector` versions or sources.

## Next dependency chain

1. Land the dependency/security baseline described in the dated PR review.
2. Converge the skillkit and AnyDoc on one exact `pdf-inspector 1.14.2`.
3. Add the provider-neutral document contract while preserving all PDF APIs.
4. Add the supervised AnyDoc worker for allowlisted non-PDF formats.
5. Add the three generic MCP tools and real, public integration fixtures.
6. Promote formats only after corpus, resource, privacy, and rollback gates
   pass.

Steps 2–6 are specified in
[`docs/anydoc-integration-plan.md`](anydoc-integration-plan.md). Do not skip
parser convergence or expose upstream AnyDoc model types directly through MCP.
