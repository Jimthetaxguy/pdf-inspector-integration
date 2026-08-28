# Upstream drift audit — 2026-08-28

## Alignment result

The parser-convergence update is implemented on this branch. The workspace uses released Firecrawl dependencies and the additive document surface now includes Linux-memory-gated strict CSV and strict EPUB routes; full locked verification is recorded below.

## Initial audit decision (superseded by alignment result)

The controlled alignment is implemented on this branch; no upstream source was vendored, and the provider-neutral contract keeps PDF routing dedicated while the enabled document lanes remain bounded.

The local checkout is clean and aligned with `origin/main` at
`f5c4859cf952a8b74d1bb0b83bc97fc9986c1fae`. The existing PDF MCP baseline
passes its locked Rust checks. Upstream has materially advanced, but the
available changes are not a low-risk maintenance bump:

- `pdf-inspector` has moved from a Git-pinned crate reporting version `0.1.0`
  to release `1.17.0`, adding optional OCR/vision, new extraction/layout
  behavior, and a newer `lopdf` dependency.
- AnyDoc `0.2.4` is now resolved in this workspace for parser convergence, adding typed scanned-page/OCR reporting without exposing a runtime path.
- AnyDoc `main` is currently at the same `0.2.4` package version and contains
  post-release README-only commits in the observed range.

The correct next implementation is an isolated parser-convergence branch, not
an immediate update on the public default branch.

## Post-alignment baseline

| Area | Observed state |
|---|---|
| Local branch | `agent/codex-align-firecrawl-20260828`, alignment changes pending commit |
| Remote alignment | `HEAD == origin/main`, zero ahead/behind |
| Workspace | `pdf-inspector-skillkit` plus `pdf-inspector-mcp` |
| MCP surface | 16 tools: 13 existing PDF/domain helpers plus capability, classification, and bounded DOCX/PPTX/XLSX/ODS/ODT/ODP/EPUB/CSV conversion tools |
| PDF dependency | Released crates.io `pdf-inspector 1.17.0`; checksum recorded in `Cargo.lock` |
| PDF transitive parser | crates.io `lopdf 0.42.0` |
| AnyDoc dependency | Exact released 0.2.4; DOCX/PPTX/XLSX/ODS/ODT/ODP/EPUB use the bounded AnyDoc worker; CSV uses a separate Linux-memory-gated local adapter; other variants remain disabled |
| MCP dependencies | `rmcp 3.1.4`, `schemars 1.2.2` after the latest local fast-forward |
| MSRV file | Package manifests declare Rust `1.88`; no repository `rust-toolchain.toml` pins a toolchain |
| Fixtures | Tracked public U.S. Code PDF corpus, public AnyDoc PPTX/ODS/ODP fixtures, synthetic DOCX/XLSX/ODT/CSV packages, generator-backed EPUB qualification corpus, and malformed/incomplete office negatives |
| Existing checks | Locked cargo check, cargo test (97 host tests), Linux-target check and Clippy, release worker evidence, and public-hygiene checks pass after the ODP/EPUB slices |

## Ownership and divergence map

| Local area | Classification | Upstream relationship |
|---|---|---|
| `crates/pdf-inspector-skillkit/src/lib.rs` | Local wrapper | Re-exports selected upstream types, validates/canonicalizes paths, serializes results, and maps errors. |
| `crates/pdf-inspector-skillkit/src/domain/` | Local extension | Tax, IRC, SEC, and Sweet logic has no equivalent in the Firecrawl parser repositories. |
| `crates/pdf-inspector-mcp/src/main.rs` | Local integration | MCP schemas, tool registration, timeout handling, and path-free response/error policy are local. |
| `test-corpus/` and integration tests | Local verification | Public fixture and domain acceptance layer; not upstream test corpus. |
| PDF parser internals | External dependency | Resolved from Firecrawl `pdf-inspector`; no source copy exists locally. |
| AnyDoc worker/router contract | Versioned stdin/stdout worker, stable local types, sanitizer, OOXML/ODF preflight, omission diagnostics, strict DOCX/PPTX/XLSX/ODS/ODT/ODP/EPUB gates, and Linux-memory-gated strict CSV adapter; upstream types remain private. |

## `pdf-inspector` drift

The local revision is the April 2026 detector fix at `2f23f07f`. The upstream
`v1.14.2` release is `4bee4f9`; `v1.15.0` is `06a9bab6`; current `main` is
`23cf1ad7`. The `v1.15.0` release range contains substantial changes rather
than a narrow patch update:

- Optional OCR/vision pipeline, PDFium, OAR/ONNX, and model-cache/download
  features.
- Markdown and layout fixes, including hyphenation and table heuristics.
- New public `vision` surface and binding/CLI changes.
- `lopdf` changes from the locally pinned Git `0.40.0` line to released
  `0.42.0` in the upstream manifest.

Current `main` advances further through `v1.17.0` with RTL geometry handling,
table recovery, layout segmentation, and release/CI changes. The local wrapper
calls symbols still present by inspection (`process_pdf`, `detect_pdf`, memory
variants, region extraction, `PdfProcessResult`, `PdfOptions`, and related
types), but source presence is not sufficient evidence of output compatibility.

Disposition: alignment is accepted after the isolated branch compiled and all 13 existing tools passed; retain the no-OCR-default boundary pending a separate contract and resource/privacy gate.

## AnyDoc drift

AnyDoc `v0.2.4` is now resolved in the workspace at
`42bf1c5ecdde9eb0d96d6bd75a9e6698cf93b14c`. Release `v0.2.4` is
`42bf1c5e` and changes the complete-conversion contract for PDFs:

- Scanned or image-only pages now produce typed `ConvertError::NeedsOcr`
  with page numbers and total page count instead of silently dropping pages.
- New fixtures cover mixed and scanned PDFs.
- Node/Python/CLI hosted OCR behavior is optional upstream functionality and
  is outside this repository’s offline MCP boundary.

The core Rust API remains `to_markdown`, `to_markdown_bytes`, and
`to_document`; the shared `Document` model still lacks the local MCP contract’s
stable provenance, warning, capability, and egress-sanitization guarantees.
Upstream still emits document-controlled log text for recoverable skipped
parts, sheets, slides, chapters, and relationships.

Disposition: released AnyDoc 0.2.4 is adopted behind the supervised worker/router contract for DOCX, exact PPTX, exact XLSX, exact ODS, exact ODT, exact ODP, and strict EPUB. The separate local strict CSV adapter is Linux-memory-gated because upstream CSV materializes rows and skips unreadable records; the upstream CSV/RTF paths remain unexposed. ODP and EPUB retain independent strict preflight and Linux memory gates.

## Pre-alignment candidate decision table

| Candidate | Decision | Required evidence before adoption |
|---|---|---|
| Pin `pdf-inspector = 1.17.0` from crates.io | Deferred | Isolated compile, MSRV check, dependency/source/license audit, PDF output regression, and MCP smoke tests. |
| Pin `pdf-inspector = 1.15.0` or `1.16.0` | Deferred | Same gates; no evidence currently favors an intermediate release over `1.17.0`. |
| Add AnyDoc `0.2.4` now | Rejected for this baseline | No worker, router, typed local contract, completeness enforcement, or public office-format corpus exists yet. |
| Import AnyDoc `main` commits | Rejected | Observed post-release commits are documentation-only; no production benefit identified. |
| Copy upstream source into this repository | Rejected | Violates the dependency-first lineage policy and adds synchronization/license burden. |

## Post-alignment next implementation sequence

1. Completed: isolated branch created from clean `main` and the aligned dependency changes applied.
2. Completed: `pdf-inspector 1.17.0` compiled against the current wrapper on Rust `1.96.0`.
3. Completed: locked metadata/tree checks show one converged parser line; provenance and handoff documents were updated.
4. Completed: all existing PDF/domain tests and strict Clippy pass under the aligned graph.
5. Completed: implement the provider-neutral AnyDoc document contract while preserving all existing PDF APIs.
6. Completed for DOCX, strict PPTX, strict XLSX, strict ODS, strict ODT, strict ODP, and strict EPUB paths: add the supervised worker, public fixtures, typed result, resource/timeout controls, OOXML/ODF/EPUB preflight, declared-part completeness validation, variant negotiation, and Markdown egress sanitization. Broader adversarial corpus, filesystem, and cross-platform promotion evidence remain next.

## ODS implementation decision

The reconciled source review selected strict ODS as the next additive format.
SOL identified tax and audit use cases including trial balances, general-ledger
exports, depreciation schedules, reconciliations, control matrices, and
checklists. MiniMax independently confirmed that ODF has stronger bounded
parser primitives than CSV/RTF, while requiring fail-closed handling for
recovery and external-content behavior.

Source evidence: AnyDoc v0.2.4 release commit
42bf1c5ecdde9eb0d96d6bd75a9e6698cf93b14c, upstream main
261fc257d17c3eab0f673be31c408fd9fdc2171a, parser modules
src/formats/odf/mod.rs and src/formats/odf/table.rs, and upstream ODS fixtures
sheet.ods, handmade-gaps.ods, and handmade-durations.ods. The reviewed main
range contains no production-code delta beyond the release.

Implemented local delta:

- exact mimetype and content.xml identity;
- well-formed spreadsheet content with at least one table;
- hidden table/sheet row/column, external xlink, encrypted manifest, active
  object/script, and uncached-formula rejection;
- worker code 4 using the existing protocol v2, 50 MiB input, 8 MiB output,
  15-second timeout, concurrency, sanitizer, and platform memory policy;
- real MCP conversion of the public two-sheet typed-value/merged-span fixture.

ODS remains an additive route only. Strict ODP and strict EPUB are Linux-memory-gated additive routes under separate presentation and EPUB 3 contracts; local preflight prevents the pinned AnyDoc parser from converting partial EPUBs as complete. The local strict CSV adapter is Linux-memory-gated, while the upstream CSV parser, RTF, and legacy/macro Office formats remain deferred under separate resource and active-content threat models.

## ODT implementation decision

The reconciled source review selected strict ODT as the next additive slice
because AnyDoc already exposes ODF text conversion through its public Format,
to_markdown_bytes, and document APIs, while the format fits the existing ZIP/XML
worker boundary. The local contract does not treat an AnyDoc success return as
proof that every ODT part was represented.

Source evidence: AnyDoc v0.2.4 at 42bf1c5ecdde9eb0d96d6bd75a9e6698cf93b14c,
upstream main at 261fc257d17c3eab0f673be31c408fd9fdc2171a, and
src/formats/odf/mod.rs plus src/formats/odf/text.rs. SOL identified research,
tax, and control-workpaper use cases. MiniMax identified skipped styles/assets,
external links, annotations/tracked changes, active content, and archive/XML
resource abuse as the relevant risks. The current main range contains no
production-code or manifest change beyond v0.2.4.

Implemented local delta:

- worker code 5 and exact ODT routing through the existing protocol v2;
- exact ODT mimetype/content identity, balanced XML, manifest encryption
  detection, and required office:text content;
- hidden/tracked/annotation/conditional content, external references, active
  forms/scripts/objects, and missing internal assets fail closed;
- public synthetic positive and negative ODT fixtures with SHA-256 manifest
  entries and real MCP worker tests;
- no Cargo.toml or Cargo.lock change, no vendored source, and no PDF route change.

Disposition: strict ODT is enabled for the narrow visible-text contract. Strict ODP and strict EPUB are enabled only on Linux under their separate presentation contracts; RTF and legacy or macro-enabled Office variants remain deferred; the upstream CSV parser remains unexposed. The next evidence dependency is broader adversarial and platform resource promotion, not an upstream dependency update or main-branch import.

## ODP implementation decision

The current source review selected strict ODP as the next additive capability
for tax/control walkthroughs, training material, and process reviews. AnyDoc
v0.2.4 exposes ODF presentation conversion through Format::Odp and
to_markdown_bytes; src/formats/odf/mod.rs handles slide text, tables, grouped
shapes, images, and speaker notes. The refreshed AnyDoc main
261fc257d17c3eab0f673be31c408fd9fdc2171a contains no production-code or
manifest delta after release v0.2.4, so no dependency update is required.

The local implementation adds worker code 7 and a strict package preflight:
exact presentation mimetype, content.xml, balanced XML, office:presentation
with at least one page, local referenced assets, and rejection of active,
external, hidden, encrypted, malformed, and archive-amplified input. The
public fixture and eight deterministic derivatives are recorded in the corpus
manifest. The private real worker converted the public fixture on Darwin and
the expected rejection matrix passed. Public MCP promotion is Linux-only
because the worker address-space ceiling is enforceable there.

The SOL specialist did not return a usable memo after bounded waits and the
MiniMax transport was unavailable. This is recorded as missing review input,
not as approval. The decision is therefore constrained to the live source
inspection and the existing promotion policy. No upstream source was copied,
no Cargo dependency changed, and PDFs remain on the dedicated pdf-inspector
route. ODP non-goals are layout fidelity, animation semantics, source
coordinates, embedded-object execution, and external fetching.

## Deferred-format review and current promotion evidence

The ODP sidecar review attempt in this iteration produced no usable SOL or
MiniMax memo (SOL timed out and the MiniMax transport closed), so no specialist
approval is inferred. Direct inspection of AnyDoc v0.2.4 and refreshed `main`
found no production parser or dependency delta requiring an upstream merge.

ODP is now a separate local strict lane. It requires exact package identity,
balanced presentation XML, at least one visible declared page, local asset
resolution, and rejection of active, external, hidden, encrypted, malformed,
and archive-amplification inputs. The real public fixture and deterministic
adversarial derivatives are recorded in `test-corpus/odp/`; Darwin worker tests
provide parser/output evidence, while MCP enablement is Linux-only because the
address-space ceiling is enforceable there.

Strict EPUB is now an additive Linux-memory-gated route. Its local preflight
requires exact EPUB 3 OCF identity, one OPF rootfile, all declared spine targets,
navigation agreement, local resources, and rejection of active, external, hidden,
encrypted, malformed, and archive-amplified inputs. The real worker proves
chapter-order output and the public corpus exercises the negative matrix. The
upstream CSV and RTF frontends remain unexposed; strict CSV is a separate local
adapter with its own bounded contract. Filesystem, cross-platform, and broader
hostile-input evidence remain promotion dependencies.

This is not a Cargo update or source import. The local ODP adapter is additive,
keeps PDF routing unchanged, and records its source lineage without vendoring
upstream code.

## Verification evidence and limitations

The current continuation run passed the prior locked gates; the ODP and EPUB extensions add focused unit, worker, and Linux-target compile coverage listed below.

- `cargo fmt --all -- --check`
- `cargo metadata --locked --format-version 1`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- cargo test --workspace --locked: 97 host tests passed, with Linux-only route tests additionally compiled; 0 failed, 0 ignored
- `cargo build --workspace --release --locked`
- `cargo audit`: no vulnerabilities; warnings for unmaintained `ttf-parser 0.25.1` (`RUSTSEC-2026-0192`) and yanked `chacha20 0.10.0`
- `bash scripts/check-public-hygiene.sh`
- release-binary MCP stdio smoke: 16 tools listed; `document_capabilities` returned schema version 2 and exact `.ods` cached/displayed-value-only policy; release worker evidence covers EPUB on the host, while Linux route tests compile for the enabled path and remain CI-runtime gated
- fixture archive tests and recorded SHA-256 values for DOCX/PPTX/XLSX/ODS/ODT public inputs
- Unix worker process-group termination and reap regression test
- MCP end-to-end worker-timeout test with descendant-process reap assertion
- upstream AnyDoc abuse evaluator: 7/7 reviewed DOCX/PPTX/ODS fixtures returned
  stable `resource_limit` under the Darwin release worker

`cargo-deny` is not installed on this host, so its advisory/license/source policy command remains an external CI gate. The current Darwin/arm64 resource report records initial release-mode worker observations for all enabled lanes, but Darwin still has no production process-memory ceiling or filesystem isolation; Linux has the address-space boundary and seccomp network-denial filter, while Darwin uses the named `no-network` profile. The worker-level network canary is green for its scoped Darwin/Linux paths, and broader hostile-format promotion remains gated until additional hostile-input
resource evidence, filesystem isolation, and non-Linux memory evidence are
available; the reviewed upstream abuse corpus is green for its scoped Darwin run.

## Iterative file-type review

The fresh source-backed SOL review and independent MiniMax threat/evaluation
review were reconciled against the refreshed AnyDoc 'main' mirror at
261fc257d17c3eab0f673be31c408fd9fdc2171a and the released v0.2.4 parser.

- SOL ranked PPTX first for tax/control-process walkthroughs because the parser
  preserves slide order, title/body cascades, notes, tables, links, charts, and
  assets. It also identified the critical limitation: unreadable slides are
  logged and skipped, so a local completeness gate is required.
- MiniMax proposed CSV first for bank, general-ledger, payroll, and 1099 exports,
  with injection neutralization, encoding/delimiter tests, row/field caps, and
  deterministic output. That proposal was adopted as the separate local strict CSV contract described below; the upstream AnyDoc implementation remains unexposed because issue #104 documents broad materialization and memory risk.
- Decision: strict PPTX was implemented first, followed by strict ODS and ODT
  under their narrow contracts. This cycle adds strict ODP and strict EPUB with Linux-only MCP
  enablement; the upstream CSV/RTF parsers, macro-enabled, slideshow, and legacy
  binary routes remain unexposed. No dependency update or upstream-main code
  import is required.

PPTX acceptance evidence now includes:

1. A real AnyDoc conversion through the MCP worker using the public,
   provenance-recorded inheritance fixture.
2. Exact .pptx classification with .ppsx classified but disabled and
   macro-enabled presentation variants classified as active content.
3. Package preflight that requires [Content_Types].xml,
   ppt/presentation.xml, presentation relationships, every declared slide
   target, well-formed slide XML, and a shape tree.
4. Fail-closed handling for hidden slides, external relationships, embedded
   objects, OLE/ActiveX/control parts, malformed slides, and incomplete
   declarations.
5. Existing DOCX/XLSX/PDF tests retained as regression coverage.

Known remaining limitations are explicit: the local completeness oracle is
structural rather than a typed upstream skipped-part report; source-coordinate
and slide-level citation metadata are not exposed; Linux has the enforced
address-space ceiling while macOS/non-Linux memory containment remains a
promotion gate; and cargo-deny remains an external CI gate on this host.

## CSV implementation decision

CSV is the next bounded additive lane after the prior source-backed review
identified bank, general-ledger, payroll, and 1099 exports as high-value
workflows. The live AnyDoc v0.2.4 and current-main comparison confirmed that
the upstream frontend materializes rows, infers delimiter/header behavior, and
skips unreadable records. Those semantics are incompatible with this
repository's completeness and resource contract.

The implementation therefore adds no dependency and imports no upstream source.
Worker code 6 invokes a local strict parser with valid-UTF-8/BOM handling,
deterministic delimiter sniffing, RFC-4180-style quoting, equal-width rows,
bounded rows/columns/fields/output, and Markdown escaping. It is enabled only
on Linux because the current worker has an enforceable address-space ceiling
there; other platforms classify CSV but keep the route disabled.

The public evidence is the synthetic non-PII CSV fixture, parser unit coverage,
the platform-gated classification test, and the Linux-target real MCP worker
test. A fresh specialist dispatch for correlation anydoc-iteration-20260828-epub-html-ocr
was attempted during this slice but did not return a usable SOL or MiniMax
memo; this decision relies on the prior recorded adversarial CSV recommendation
plus direct live-source inspection and keeps cross-host promotion open. No
claim is made that the upstream AnyDoc CSV parser was adopted.

## Sources

- <https://github.com/firecrawl/pdf-inspector>
- <https://github.com/firecrawl/anydoc>
- Local integration design: `docs/anydoc-integration-plan.md`
- Local public handoff: `docs/HANDOFF.md`
