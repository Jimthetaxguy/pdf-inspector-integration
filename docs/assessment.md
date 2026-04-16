# pdf-inspector — integration assessment (Part 1)

<!-- STATUS: PARTIAL — Part 1 baseline. Expand corpus in Part 3. -->

Date: 2026-04-15
Upstream pin: `firecrawl/pdf-inspector@2f23f07f` (2026-04-14)
Clone path (read-only): `~/code/third_party/pdf-inspector`
Integration workspace: `~/code/pdf-inspector-integration` (this dir)

> Shareability: this file is safe to share. Personal corpus paths are

### 5.2 Results

| Metric | Value |
|---|---|
| Files processed | 20 |
| Parse failures | 1 (5%) — path contained `|` character, shell-escape issue in test harness, not pdf-inspector itself |
| `text_based` | 19 |
| `mixed` | 1 |
| `scanned` / `image_based` | 0 |
| Median detection time | 1 ms |
| Max detection time | 9 ms (63-page doc) |
| Confidence = 1.00 | 18 / 20 |
| Confidence < 1.00 | 1 file @ 0.86, 1 file @ 0.70 (mixed) |

Extraction smoke test on a 4-page text-based PDF: `pdf2md --raw`
completed in 11ms total.

### 5.3 Observations

- Speed claim (10–50ms classification) holds — most files were under 5ms.
- Confidence scoring is granular: the 0.86 result was a ~7-page doc with
  mixed text/image content that was still legitimately readable.
- No panics or stack traces across the sweep.
- The one `mixed` result correctly identified a doc with image-heavy
  pages, confirming the per-page `pages_needing_ocr` routing primitive.

## 6. Gap list (feeds Part 3 prioritization)

Ordered by expected Part-3 value:

1. **No scanned/tax-form samples in quick corpus** — can't yet validate
   the tax-form and scanned-heuristics skills. Stratified corpus is the
   first Part-3 task.
2. **No `--help` / `--version` on CLIs** — fine for library use; Part 2
   MCP wrapper will expose a proper tool-description surface.
3. **No streaming API** — entire doc loaded into memory. For very large
   PDFs (>50MB) we'll cap input at the MCP boundary (Part 2.3).
4. **No plugin trait inside pdf-inspector** — confirmed; reinforces
   post-processor-only approach.
5. **Table detection lags OCR engines** (per upstream README: 0.59 TEDS
   vs 0.83+ for OCR/ML tools). Directly addressed by
   `pdf-financial-tables` skill in Part 3.
6. **Heading detection trails opendataloader** (0.57 MHS vs 0.74). Many
   PDFs use same-size bold for headings — post-processor can fix with
   domain knowledge (e.g. "SEC 10-K item headings always match regex
   `^Item \d+[A-Z]?\.`").
7. **lopdf git-pinned dep** — upstream pins a specific SHA on J-F-Liu's
   fork. If lopdf itself shifts, our reproducibility still holds because
   we pin pdf-inspector by SHA in Part 2's workspace.
8. **Dep tree bigger than advertised** — 9 direct deps, not 1. Minor
   supply-chain note; none are load-bearing for our threat model.
9. **Path-escaping bug in test harness** — not a pdf-inspector issue;
   noted so the Part 3 benchmark runner uses `argv` arrays, not shell
   splitting.

## 7. Part 1 deliverables — status

- [x] Binaries installed and on PATH
- [x] Upstream clone at `~/Code/third_party/pdf-inspector` (read-only)
- [x] API surface inventoried
- [x] Dependency tree captured
- [x] Extension points documented
- [x] Baseline benchmark (quick corpus, 20 PDFs) run and recorded
- [x] Gap list drafted

**Ready for Part 2 (MCP server + plugin + cross-agent mirrors).**

Upstream SHA to pin in Part 2's Cargo.toml:
`rev = "2f23f07f6e38fd341361554c114d1abe36349ce7"`
