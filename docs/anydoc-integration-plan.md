# Firecrawl AnyDoc integration plan

**Evidence date:** 2026-08-22
**Plan status:** proposed architecture; implementation has not started
**Target:** additive multi-format document support without changing the 13
existing PDF/domain MCP tools

## Decision

Integrate Firecrawl AnyDoc as an exact-version Rust dependency behind the
skillkit boundary. Keep every PDF on the existing PDF adapter. Expose
allowlisted non-PDF formats through a supervised, killable worker built from
the same Rust workspace.

No non-PDF format is enabled merely because `AnyDoc::to_markdown` returns
`Ok`. Version 0.2.3 can recover from skipped package parts and report the event
only through document-controlled log text. Each parser path therefore remains
disabled until completeness is observable through a typed signal or the
conversion fails closed.

Do not use the Node CLI, `npx`, Python bindings, WASM wrapper, Docker, or the
hosted Firecrawl Parse API. The local library path is offline and does not need
an API key. The hosted API uploads documents and is a different privacy and
authorization boundary.

CSV and RTF remain disabled in the first release because upstream issue
[#104](https://github.com/firecrawl/anydoc/issues/104) documents memory
exhaustion that a 50 MiB consumer input cap only partially mitigates.

## Verified upstream baseline

| Property | Observed evidence |
|---|---|
| Repository | [`firecrawl/anydoc`](https://github.com/firecrawl/anydoc) |
| Version and revision | `v0.2.3` at signed commit `bf3d33e61731580d1ee1c6a85e56093d715a21a6` |
| License | MIT |
| Core runtime | Rust 2024; minimum supported Rust `1.88` |
| Public Rust API | `to_markdown`, `to_markdown_bytes`, `to_document`, and content/extension format detection |
| Local network behavior | No HTTP client, telemetry, update checker, API-key lookup, external service, or ML model in the resolved local runtime |
| Formats | Word, PowerPoint, Excel, OpenDocument, RTF, EPUB, CSV, and PDF variants |
| PDF behavior | Delegates to `pdf-inspector 1.14.2`; PDFs bypass AnyDoc's shared document model; image-only PDFs require OCR |
| Safety controls | Fixed archive, decompression, XML, table, binary-record, and retained-asset limits |
| Upstream tests | 294 root-crate tests passed and 1 was ignored at the pinned revision; checked-in fixtures and fuzz targets exist, but fuzz targets are not exercised by upstream CI |

The upstream `Document` model is not a suitable MCP contract. It lacks Serde
schemas, provider/version metadata, stable warnings, page/slide/source
coordinates, and a PDF representation. Embedded assets contain raw bytes and
must not be serialized through MCP by default.

Upstream benchmark results are directional only: the 100-document corpus is
not redistributable, PDFs are excluded, and the quality judge is not locally
reproducible. This repository needs its own public acceptance corpus.

## Why the initial sidecar assumption changed

The earlier QA review correctly asked for isolation if AnyDoc were a different
runtime. Live inspection shows that it is a native Rust library, so a remote
service or language sidecar would add needless supply-chain and deployment
surface.

Process isolation is still required at the MCP execution boundary. Tokio
cannot abort a `spawn_blocking` closure after it starts; dropping the join
handle only stops waiting for it. A response timeout therefore does not prove
that a pathological parser stopped consuming CPU or memory. The worker below
is a small Rust binary using the same library, not an external service.

## Target architecture

```text
                                  existing 13 tools
                                         |
                                         v
                                +------------------+
                                |    PDF adapter   |
                                +--------+---------+
                                         |
                                         v
MCP stdio  ->  document service/router  ->  pdf-inspector =1.14.2
                     |
                     | allowlisted non-PDF input bytes
                     v
             supervised anydoc-worker
             (workspace Rust binary)
                     |
                     v
               AnyDoc adapter
               anydoc =0.2.3
                     |
                     v
          completeness + output sanitizer
```

Both adapters live behind `pdf-inspector-skillkit`. Production MCP conversion
uses the worker for non-PDF parsing; contract tests may invoke the real adapter
in-process. The worker receives document bytes on stdin, never a shell command
or user path, and returns Markdown on bounded stdout.

## Deterministic routing policy

Routing uses content signatures first. An extension may resolve a
signature-less format only when that format is enabled. There is no
try-every-parser chain.

| Input | Route | Initial status | Reason |
|---|---|---|---|
| PDF | Existing PDF adapter | Enabled | Preserves classification, page/OCR diagnostics, regions, and domain helpers |
| DOC/DOCX/DOCM | AnyDoc worker | Gate-controlled | Real Rust parser; macros are not executed |
| PPT/PPS/POT/PPTX/PPTM/PPSX/PPSM | AnyDoc worker | Gate-controlled | Markdown only; slide-boundary limitations reported as capability metadata |
| XLS/XLSX/XLSM/XLSB | AnyDoc worker | Gate-controlled | Markdown only; source-coordinate limitations reported as capability metadata |
| ODT/ODS/ODP | AnyDoc worker | Gate-controlled | Allow after public fixture tests |
| EPUB | AnyDoc worker | Gate-controlled | Parser does not fetch relationships; output egress still requires sanitization |
| CSV | None | Disabled | Upstream #104 memory-exhaustion risk; no content signature |
| RTF | None | Disabled | Upstream #104 memory-exhaustion risk |
| Image-only or scanned PDF | None | Typed `ocr_required` error | No silent upload or hosted OCR fallback |
| Unknown, encrypted, oversized, or disallowed input | None | Typed hard error | Never fall back after a security or permission failure |

No existing PDF tool changes its route. The new generic Markdown tool routes a
PDF through the existing adapter, not through AnyDoc's PDF wrapper.

An AnyDoc success value is not evidence of a complete conversion. At the
pinned version, EPUB chapters, presentation slides, spreadsheet sheets, and
archive parts can be skipped while the call still succeeds. Upstream log text
is neither a public warning schema nor safe to forward. A format stays disabled
unless an audited, typed completeness mechanism covers every recoverable skip
path; any detected skip becomes `incomplete_conversion`.

Encrypted OOXML commonly uses an OLE container with `EncryptedPackage` and
`EncryptionInfo` streams, for which AnyDoc content detection can return
unknown. Run an explicit encryption preflight before extension fallback so the
router returns `encrypted` rather than `unsupported` or selecting a parser by
filename alone.

## Provider-neutral contract

Keep upstream types private. Define local, serializable types with an explicit
wire schema version:

- `DocumentKind`: stable format family and optional container variant.
- `DocumentProvider`: name, exact version, source revision/checksum, and local
  execution class.
- `DocumentCapabilities`: enabled formats, disabled reason, OCR availability,
  structured-model availability, diagnostic availability, and input/output
  limits.
- `DocumentClassification`: detected kind, confidence/diagnostics when the
  provider actually supplies them, and selected route.
- `DocumentContent`: Markdown, byte count, provider chain, warnings, and
  `schema_version`.
- `DocumentWarning`: typed diagnostics only. Do not turn AnyDoc's unstable log
  text into a public schema. Local sanitizer warnings such as
  `remote_asset_removed` are allowed because this repository owns them.
- `DocumentError`: stable codes for not found, not regular file, too large,
  unsupported, disabled format, malformed, encrypted, resource limit,
  incomplete conversion, timeout, worker failure, output too large, and OCR
  required.

Provenance records the full provider chain. A PDF result identifies
`pdf-inspector`; an office result identifies `anydoc`. Do not claim page,
slide, sheet, or source-offset provenance that upstream does not expose.

## Operational controls

- Canonicalize the input and require a regular file before reading it.
- Retain the current 50 MiB input cap unless corpus measurements justify a
  smaller per-format cap.
- Send bytes to the worker through stdin so it has no ambient input-path
  authority.
- Spawn with an argv array, never a shell; clear the environment and add only
  the minimum runtime variables.
- Install a sink logger before invoking AnyDoc. Upstream log messages can
  contain document-controlled sheet names, URLs, internal part names, cell
  values, and formatting values; never forward or persist them.
- Map upstream errors by audited variant to constant local error codes. Never
  include upstream `part`, `detail`, `Display`, or debug strings in stdout,
  stderr, telemetry, or MCP responses.
- Keep stdout reserved for the versioned IPC envelope and stderr for the
  worker's own bounded, path-free structured events.
- Stop reading and kill the worker if stdout exceeds 8 MiB. Return
  `output_too_large`; never silently truncate document content.
- Kill and reap the worker/process group when the 30-second deadline expires.
- Establish an OS memory ceiling from measured public/adversarial fixtures,
  with an absolute ceiling of 1 GiB. A platform without an enforceable memory
  boundary cannot enable untrusted CSV or RTF.
- Put a global semaphore and aggregate byte/memory budget above worker launch.
  Per-process limits alone allow concurrent workers to exhaust the host.
- Treat the worker as crash and resource containment, not a hostile-input
  sandbox: a same-UID subprocess otherwise retains ambient filesystem and
  network authority. Close inherited file descriptors, use an empty private
  working directory, and deny filesystem/network access per platform. If those
  controls cannot be enforced, narrow the supported threat model and leave
  untrusted inputs disabled.
- Do not expose embedded asset bytes in the first MCP schema.
- Parse returned Markdown into an AST, remove raw HTML, and neutralize remote,
  `file:`, and `data:` image destinations before returning content. Parser
  no-network behavior does not prevent a rendering client from fetching a
  preserved image URL.
- Treat any observable skipped chapter, slide, sheet, relationship, or package
  part as `incomplete_conversion`; never return partial content as success.
- Do not log paths, document text, embedded URLs, or filenames. Use an opaque
  request identifier.
- Verify the resolved production graph contains no HTTP client or two
  `pdf-inspector` sources.

## Dependency-ordered implementation gates

### Gate 0 — dependency, CI, and public baseline

Land the work described in
[`dependency-pr-review-2026-08-22.md`](dependency-pr-review-2026-08-22.md):

- consolidate the current direct dependency updates;
- update `crossbeam-epoch` beyond `RUSTSEC-2026-0204`;
- make cargo-deny actually check advisories and sources;
- replace home-directory fixture discovery with tracked public fixtures;
- remove machine-specific paths and potentially identifying pilot references
  from the tracked tree;
- record public fixture provenance.

**Exit evidence:** full Rust gates, RustSec scan, secret/PII scan, and a clean
tracked tree.

**Candidate-branch status:** the code and CI changes are present on this branch,
but Gate 0 is not satisfied for the public default branch until the branch is
pushed, its new CI jobs pass on that exact head, and the change is merged.

### Gate 1 — parser convergence

Replace the old Git-source `pdf-inspector 0.1.0` with exact
`pdf-inspector = "=1.14.2"`, then add exact `anydoc = "=0.2.3"`. Commit the
Cargo lockfile checksums and record the signed upstream revisions in
`THIRD_PARTY.md`.

Update skillkit calls for the current PDF API before adding document tools.
Declare and test the workspace MSRV required by AnyDoc (`1.88`). Resolve or
explicitly track the transitive `ttf-parser 0.25.1` unmaintained advisory with
an owner, upstream reference, and review trigger; parser convergence alone
does not remove that warning.

Run all resolution/build gates with `--locked`. Keep `unknown-git = "deny"`,
require revision-pinned Git sources, and delete source allowlist entries when
convergence removes the corresponding Git dependencies.

**Exit evidence:** `cargo metadata` and `cargo tree -d` show exactly one
`pdf-inspector 1.14.2`; all 13 existing tools compile and their public PDF
fixture outputs remain compatible.

### Gate 2 — contract plus unchanged PDF adapter

Add the local contract and router. Wrap the existing PDF functions as the PDF
provider without changing current MCP names or schemas. Add contract tests for
error mapping, provenance, capability truthfulness, PDF route selection, and
encrypted OOXML preflight before extension-based routing.

**Exit evidence:** existing PDF and domain tests remain green; new router tests
prove every PDF takes the PDF adapter and every hard failure prevents fallback.

### Gate 3 — supervised AnyDoc worker

Add a workspace-built worker and the real AnyDoc adapter. Enable only the
allowlisted office/OpenDocument/EPUB formats. Use raw stdin bytes, bounded
stdout/stderr, typed exit mapping, process-group cleanup, timeout termination,
an enforceable memory boundary, a global concurrency budget, and a versioned
IPC handshake. Resolve the exact sibling worker executable; never search
`PATH`.

The worker must suppress all upstream logs, map upstream errors without their
document-controlled fields, fail on observable incomplete conversions, and run
the Markdown/HTML egress sanitizer before returning content. If the pinned
AnyDoc code has a recoverable skip path without a typed completeness signal,
that parser path remains disabled until upstream adds one or a minimal audited
patch exposes it.

Tests must exercise the real AnyDoc library with public fixtures. Fakes are
permitted only in test files for worker failure injection; they do not satisfy
the integration gate. Hostile-input support additionally requires per-platform
filesystem/network denial; a killable subprocess alone is not a sandbox.

**Exit evidence:** real conversion tests, malformed/encrypted/incomplete/
resource-limit tests, remote-image and raw-HTML sanitizer tests, output-cap
tests, worker-kill tests, aggregate-concurrency tests, and a no-network
dependency/runtime check all pass. CSV and RTF remain explicitly disabled.

### Gate 4 — small backward-compatible MCP surface

Add only:

- `document_capabilities`
- `classify_document`
- `document_to_markdown`

`document_capabilities` reports compiled and runtime availability, disabled
formats, provider versions, limits, and diagnostic gaps. `classify_document`
detects kind and route; it must not imply OCR/classification detail that the
provider cannot produce. `document_to_markdown` returns the local contract.

**Exit evidence:** the exact old 13-name set remains present, the three new
names are additive, schemas are versioned, and an MCP stdio smoke test calls a
real non-PDF fixture through the worker.

### Gate 5 — public corpus and promotion policy

Create at least one synthetic or redistributable fixture for every enabled
parser/container path, with source/generator, redistribution basis, SHA-256,
transformation, and metadata-scrub status in the manifest. Legacy DOC, PPT, and
XLS; OOXML DOCX/PPTX/XLSX; XLSB; and each ODF container take different code
paths and cannot share one family-level acceptance fixture. Advertise only the
extensions whose exact route has positive and adversarial coverage.

For every enabled family, require:

- correct deterministic kind detection;
- non-empty Markdown containing fixture-specific sentinel content;
- stable table/list/heading assertions relevant to that format;
- typed failures for malformed, encrypted, oversized, and disallowed inputs;
- no path or document-content leakage in logs/errors;
- no outbound network activity;
- no remote image destinations or raw HTML capable of renderer egress;
- a hard incomplete-conversion failure for every exercised skip path;
- enforced response, output, and worker resource bounds;
- successful rollback to the PDF-only build.

Published upstream benchmark scores are not acceptance evidence for this gate.

### Gate 6 — documentation and rollout

Update README, CHANGELOG, CONTEXT, THIRD_PARTY, contributor guidance, and MCP
examples. New generic tools may become available only after Gates 0–5 pass.
PDF routing never changes by default. CSV/RTF require either closure of
upstream #104 with verified bounds or a separately proven OS-sandbox policy.

## Planned file map

| File | Change |
|---|---|
| `Cargo.toml` | Exact AnyDoc/PDF versions, workspace MSRV policy |
| `crates/pdf-inspector-skillkit/Cargo.toml` | AnyDoc dependency |
| `crates/pdf-inspector-skillkit/src/document/mod.rs` | Public local contract entry point |
| `crates/pdf-inspector-skillkit/src/document/model.rs` | Serializable contract and provenance |
| `crates/pdf-inspector-skillkit/src/document/error.rs` | Stable error taxonomy |
| `crates/pdf-inspector-skillkit/src/document/pdf.rs` | Existing PDF adapter |
| `crates/pdf-inspector-skillkit/src/document/anydoc.rs` | Private AnyDoc type/error mapping |
| `crates/pdf-inspector-skillkit/src/document/router.rs` | Deterministic content-based route policy |
| `crates/pdf-inspector-mcp/Cargo.toml` | Explicit `[[bin]]` declaration for the worker |
| `crates/pdf-inspector-mcp/src/bin/anydoc_worker.rs` | Versioned stdin/stdout worker binary |
| `crates/pdf-inspector-mcp/src/main.rs` | Supervisor, sibling-binary resolution, and three additive MCP tools |
| `crates/pdf-inspector-skillkit/src/document/sanitize.rs` | Markdown AST and raw-HTML egress sanitizer |
| `crates/pdf-inspector-skillkit/tests/document_integration.rs` | Real public-format contract tests |
| `test-corpus/` | Manifest-listed multi-format fixtures and golden Markdown assertions |

The precise module split may be collapsed if individual files remain small,
but ownership boundaries and public types must remain unchanged.

## Verification commands

```bash
cargo fmt --all -- --check
cargo metadata --locked --format-version 1
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo audit
cargo deny --locked check advisories licenses bans sources
cargo tree -d
```

Also run an MCP stdio initialization/tool-list/call smoke, a worker termination
test, fixture hash validation, Gitleaks against the tree and reachable history,
and a tracked-text scan for home paths, emails, credentials, and identifier
patterns. Do not claim the integration ready from a compile-only check.

## Rollback

Rollback is additive and explicit:

1. Disable/remove the three generic MCP registrations.
2. Remove the worker binary and AnyDoc adapter dependency.
3. Retain the converged, verified PDF dependency only if its legacy regression
   suite remains green; otherwise restore the previous lockfile and revision.
4. Re-run the exact 13-tool stdio smoke and public PDF corpus.

No migration changes stored documents or external state, so rollback does not
require data conversion.

## Non-goals

- Replacing or renaming the existing 13 tools.
- Sending documents to the hosted Firecrawl Parse API.
- OCR, image understanding, macro execution, or embedded-object extraction.
- Exposing AnyDoc's raw `Document` model or asset bytes through MCP.
- Silent parser fallback, silent truncation, or version drift.
