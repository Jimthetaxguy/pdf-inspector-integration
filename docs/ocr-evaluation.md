# OCR evaluation — Phase 4.A research spike

<!-- STATUS: COMPLETE — research report for Phase 4.A. Phase 4.B deferred. -->

**Date:** 2026-04-15
**Scope:** Phase 4.A of Part 4 (OCR tier) — research only, no production code
**Companion:** [`anydoc-integration-plan.md`](anydoc-integration-plan.md)

---

## Question

pdf-inspector closes the "text-based PDF" branch of our F1 routing
framework. What's the best Rust-based OCR engine to close the **scanned
PDF** branch, matching our single-binary, offline, no-FFI deployment
model?

## TL;DR

**Candidate: [ocrs](https://github.com/robertknight/ocrs)** (robertknight).
Pure Rust via RTen (own ONNX runtime), with an observed public-fixture baseline
of roughly 2 seconds per scanned page at 300 DPI and 100% recall under this
spike's limited keyword metric across three U.S. Code fixtures. This is not a
production-quality evaluation.

**Runner-up: `ocr-rs` / `paddle-ocr-rs`** — library-only, no CLI ships
by default. Requires a thin wrapper to benchmark, which is Phase 4.B
work, not research-spike work. Defer evaluation until we actually need
multilingual accuracy or a second opinion on ocrs's outputs.

**Research recommendation:** Keep ocrs as the first candidate if an OCR phase
starts, then evaluate it against the public corpus, resource, privacy, and
rollback gates before shipping it.

---

## Test corpus

3 synthetic-scanned PDFs, each constructed by:
1. Selecting a public U.S. Code source PDF
2. Rendering page 1 at 300 DPI via `pdftoppm`
3. Wrapping the PNG into an image-only PDF via PIL (no text layer)

All 3 synthetic scans classify as `pdf_type: "scanned"` with `confidence:
0.95` in pdf-inspector — confirming they are indistinguishable from real
scans for routing purposes.

| Sample | Source | Pages (orig) | PNG size | Domain |
|---|---|---|---|---|
| sample-1 | USC26 Subchapter V (Title 11 Cases) | 4 | 769 KB | Tax law |
| sample-2 | USC26 Chapter 2 (Self-Employment Income) | 35 | 870 KB | Tax law |
| sample-3 | USC26 Chapter 6 (Consolidated Returns) | 27 | 702 KB | Tax law |

Full scripts: `scripts/build-scanned-corpus.py`, `scripts/ocr-bench.py`.

## Engines considered

Filter criteria (from Part 4 plan): pure Rust, no C/C++ system deps,
cross-platform, single binary.

| Engine | Status | Evaluated? | Why / why not |
|---|---|---|---|
| **ocrs** | Pure Rust CLI ready via `cargo install ocrs-cli` | ✅ Yes | Primary candidate |
| ocr-rs (aka rust-paddle-ocr) | Library only, no CLI | ❌ Deferred | Requires a separate benchmark wrapper; outside Phase 4.A |
| paddle-ocr-rs | Library only, via ONNX Runtime | ❌ Deferred | Same — library only |
| tesseract / tesseract-rs | Requires system C++ deps | ❌ Skipped | Violates Part 2 deployment model |
| extractous | Wraps Apache Tika via GraalVM | ❌ Skipped | Not actually Rust in practice |
| Ferrules | macOS-only | ❌ Skipped | Fails cross-platform requirement |

Evaluating `ocr-rs` requires building a benchmark CLI on top of its library
API, which is outside Phase 4.A's dependency boundary. If ocrs proves
insufficient, that wrapper is the next comparison path.

## ocrs benchmark results

Command: `ocrs <sample>.png` (default settings, no preprocessing).

| Sample | Time (s) | OCR chars | Accuracy ratio | Keyword recall |
|---|---|---|---|---|
| sample-1 | 1.89 | 2,871 | 0.948 | 1.000 |
| sample-2 | 2.03 | 3,471 | 0.588 | 1.000 |
| sample-3 | 1.62 | 2,655 | 0.848 | 1.000 |

**Metric notes:**
- **Accuracy ratio** = `difflib.SequenceMatcher.ratio()` between
  normalized OCR output and the first 3,000 chars of ground truth
  markdown. This is a **rough** signal — it compares alignment, not
  just presence. Low values here don't mean "OCR failed" — they often
  mean "ground truth covers more than just page 1" (e.g. sample-2 has
  35 pages, so the first 3,000 characters of full-document Markdown cover a small
  slice of the scan).
- **Keyword recall** = fraction of the 50 most-frequent content words
  from ground truth that appear in OCR output. This is the **reliable**
  signal. All three retained public fixtures measured 100% under this
  benchmark's limited keyword set; that is not a production-quality claim.

### Observed errors

| Error type | Typical impact |
|---|---|
| Glyph confusion | Visually similar letters may be substituted |
| Missing separators | Table or column boundaries may be lost |
| Number formatting | Punctuation can change during recognition |
| Missing heading syntax | Plain OCR text does not preserve Markdown structure |

These observations are directional. The retained public corpus is too small to
support a production-readiness claim, and OCR remains outside the current
AnyDoc integration scope.

## Install & deployment footprint

| Component | Size | Notes |
|---|---|---|
| `ocrs` binary | 8.0 MB | Installed to `~/.cargo/bin` via `cargo install ocrs-cli` |
| text-detection.rten model | 2.4 MB | Auto-downloaded to `~/.cache/ocrs/` on first run |
| text-recognition.rten model | 9.3 MB | Auto-downloaded on first run |
| **Total footprint** | **19.7 MB** | Binary + both models |

Install friction: `cargo install ocrs-cli` — one command. First `ocrs`
invocation downloads models from S3 (~12 MB). No system package deps,
no GPU required, no Python.

Compare to Tesseract stack: Tesseract binary (~20 MB) + Leptonica
(~5 MB) + language data per language (~10-30 MB each) + system install
complexity. ocrs is half the footprint and zero system deps.

## Integration sketch (for Phase 4.B, when triggered)

Simplest viable architecture:

```rust
// crates/pdf-inspector-ocr-mcp/src/main.rs
use rmcp::{ServerHandler, ServiceExt, ...};

#[tool(description = "Extract text from a scanned or image-only PDF via OCR")]
async fn ocr_pdf(&self, params: Parameters<PathInput>) -> String {
    // 1. Render each page to PNG via pdftoppm or similar
    // 2. For each page, invoke ocrs (via subprocess OR as library via
    //    `ocrs = "0.12"` crate dep)
    // 3. Concatenate results with page-break markers
    // 4. Return as JSON
}
```

Two Rust API paths:
- **Subprocess** — shell out to the `ocrs` binary. Simpler, slower (~50ms
  fork overhead per page), no compile-time coupling.
- **Library** — depend on `ocrs = "0.12"` crate. Faster (no fork), but
  bigger MCP binary size (+ ~30 MB for models embedded or downloaded).

Recommendation for 4.B: **subprocess** first. Fork overhead is
negligible relative to ~2 s/page OCR time. Switch to library dep only
if profiling shows the overhead matters.

## What we learned about OCR more broadly

1. **ocrs remains a candidate, not a production dependency.** The three
   retained public fixtures establish a dated, rerunnable baseline only.

2. **Character-level accuracy metrics are misleading** for this
   comparison. The retained SequenceMatcher ratios range from 59-95%
   depending on how well the "first 3000 chars of ground truth"
   happens to align with "page 1 of the scan." The keyword recall
   metric is the one to trust.

3. **Routing remains justified, but exact latency needs a new release
   benchmark.** The retained OCR measurements are 1.6-2.0 seconds per page;
   the historical text-extraction comparison is not reproducible from the
   current public benchmark manifest.

4. **OCR remains deferred.** The current AnyDoc plan returns a typed
   `ocr_required` error for scanned PDFs rather than adding an unverified
   fallback.

## Decision record

- **Phase 4.A** — complete. Report written.
- **Phase 4.B trigger** — an explicitly approved OCR scope with public
  acceptance fixtures and resource/privacy gates. The current AnyDoc plan
  returns `ocr_required` instead of adding an implicit OCR fallback.
- **Engine choice** — ocrs is the first candidate. Compare it with alternatives
  when Phase 4.B begins; this spike does not select a production engine.
- **Architecture** — undecided. Evaluate a supervised worker versus a separate
  MCP server against the same containment, deployment, and rollback gates.

## Artifacts

- `scripts/build-scanned-corpus.py` — corpus construction; generated metadata
  and hashes may vary with tool versions and timestamps
- `scripts/ocr-bench.py` — benchmark harness
- `test-corpus/source/sample-{1..3}.pdf` — public U.S. Code source PDFs
- `test-corpus/scanned/sample-{1..3}.pdf` — synthesized image-only PDFs
- `test-corpus/scanned/sample-{1..3}-*.png` — page-1 renders at 300 DPI
- `test-corpus/ground-truth/sample-{1..3}.txt` — pdf-inspector markdown extraction (ground truth)
- `test-corpus/results/summary.json` — dated public-fixture benchmark snapshot

To reproduce:
```bash
cd <repo-root>
python3 scripts/build-scanned-corpus.py   # rebuild synthetic scans
python3 scripts/ocr-bench.py              # rerun benchmark
```
