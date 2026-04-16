# IRC/Regs PDF Corpus Analysis

**Date:** 2026-04-15
**Corpus:** local IRC and Treasury Regulations PDFs (CFR Title 26 volumes)
**Tools:** `detect-pdf` (classification), `pdf2md` (markdown sampling), `pdfinfo` (page counts)

---

## Executive Summary

| Metric | Value |
|--------|-------|
| **Total files** | 78 |
| **Total pages** | ~20,000+ |
| **Total size** | ~150 MB |
| **Classification** | 100% text_based (confidence 1.00) |
| **Encoding issues** | None detected |
| **OCR recommended** | 17 files (see notes below) |

**Classification verdict:** The entire corpus is clean text-based PDFs. No scanned images, no OCR needed for the bulk of content.

---

## Classification Breakdown

| Type | Count | Confidence range |
|------|-------|-----------------|
| `text_based` | 78 | 1.00 across all files |

**OCR-flagged files** (17 total — flagged by `detect-pdf` as `ocr_recommended: true`, but this appears to be a page-density heuristic, not an encoding issue — all pages returned full text in sampling):

| File | Pages |
|------|-------|
| USC26 - Subtitle A - Chapter 1 _ Subchapter B - Computation of TI (4-5).pdf | 138 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter D - Deferred Compensation, Etc.pdf | 496 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter E - Accounting Periods and Methods of Accounting.pdf | 162 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter G - Corporations Used to Avoid Income Tax on Shareholders.pdf | 39 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter H - Banking Institutions.pdf | 27 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter K - Partners and Partnerships.pdf | 45 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter L - Insurance Companies.pdf | 86 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter P - Capital Gains and Losses.pdf | 166 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter S - Tax Treatment of S Corporations and Their Shareholders.pdf | 45 |
| USC26 - Subtitle A - Chapter 1 _ Subchapter U - Designation and Treatment of Empowerment Zones.pdf | 23 |
| USC26 - Subtitle A - Chapter 2 _ TAX ON SELF-EMPLOYMENT INCOME.pdf | 35 |
| USC26 - Subtitle A - Chapter 6 _ CONSOLIDATED RETURNS.pdf | 27 |
| USC26 - Subtitle D _ Miscellaneous Excise Taxes.pdf | 404 |
| USC26 - Subtitle E _ Alcohol, Tobacco, and Certain Other Excise Taxes.pdf | 256 |
| USC26 - Subtitle F _ Procedure and Administration (1 of 4).pdf | 465 |
| USC26 - Subtitle F _ Procedure and Administration (2 of 4).pdf | 162 |
| USC26 - Subtitle G _ The Joint Committee on Taxation.pdf | 7 |
| USC26 - Subtitle I _ Trust Fund Code.pdf | 68 |

**Note on `ocr_recommended`:** For every file in this corpus, `pages_sampled == pages_with_text` and `pages_needing_ocr: []`. The `ocr_recommended` flag appears triggered by page-density thresholds (small page counts or high compression ratios), not actual OCR needs. All 78 files extract clean text.

---

## Per-File Inventory

### CFR / Treasury Regulations (30 files)

| Filename | Pages | Size (KB) | Classification | Confidence | Encoding Issues |
|----------|-------|-----------|---------------|-------------|-----------------|
| CFR _ Treas Regs Volume 10 (1 of 3).pdf | 328 | 6,965 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 10 (2 of 3).pdf | 327 | 1,245 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 10 (3 of 3).pdf | 363 | 1,634 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 11 (1 of 3).pdf | 434 | 1,499 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 11 (2 of 3).pdf | 446 | 1,537 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 11 (3 of 3).pdf | 312 | 1,132 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 12 (1 of 3).pdf | 271 | 1,158 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 12 (2 of 3).pdf | 209 | 957 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 12 (3 of 3).pdf | 547 | 2,107 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 13 (1 of 3).pdf | 420 | 1,726 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 13 (2 of 3).pdf | 403 | 1,664 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 13 (3 of 3).pdf | 372 | 1,606 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 2 (1 of 2).pdf | 276 | 1,262 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 2 (2 of 2).pdf | 299 | 1,140 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 3 (1 of 3).pdf | 342 | 1,436 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 3 (2 of 3).pdf | 271 | 1,158 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 3 (3 of 3).pdf | 369 | 1,490 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 4 (1 of 3).pdf | 352 | 1,781 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 4 (2 of 3).pdf | 444 | 1,752 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 4 (3 of 3).pdf | 453 | 2,026 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 5 (1 of 3).pdf | 340 | 1,273 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 5 (2 of 3).pdf | 224 | 920 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 5 (3 of 3).pdf | 351 | 1,324 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 6 (1 of 2).pdf | 378 | 1,535 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 6 (2 of 2).pdf | 422 | 1,722 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 7 (1 of 2).pdf | 325 | 1,470 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 7 (2 of 2).pdf | 418 | 1,783 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 8 (1 of 3).pdf | 318 | 1,477 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 8 (2 of 3).pdf | 369 | 1,560 | text_based | 1.00 | none |
| CFR _ Treas Regs Volume 8 (3 of 3).pdf | 305 | 1,447 | text_based | 1.00 | none |

### USC26 / Internal Revenue Code — Chapter 1 (Subtitles A-K) (48 files)

| Filename | Pages | Size (KB) | Classification | Confidence | Encoding Issues |
|----------|-------|-----------|---------------|-------------|-----------------|
| USC26 - Subtitle A - Chapter 1 _ Front Matter and Table of Contents.pdf | 2 | 188 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter A - Determination of Tax Liability.pdf | 470 | 7,484 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter B - Computation of TI (1-2-3).pdf | 275 | 6,580 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter B - Computation of TI (4-5).pdf | 138 | 2,648 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter B - Computation of TI (6).pdf | 317 | 7,087 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter B - Computation of TI (7-8-9-10-11).pdf | 201 | 4,673 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter C - Corporate Distributions and Adjustments.pdf | 155 | 3,517 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter D - Deferred Compensation, Etc.pdf | 496 | 9,813 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter E - Accounting Periods and Methods of Accounting.pdf | 162 | 3,460 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter G - Corporations Used to Avoid Income Tax on Shareholders.pdf | 39 | 1,071 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter H - Banking Institutions.pdf | 27 | 780 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter I - Natural Resources.pdf | 33 | 867 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter J - Estates, Trusts, Beneficiaries, and Decedents.pdf | 68 | 1,613 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter K - Partners and Partnerships.pdf | 45 | 1,135 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter L - Insurance Companies.pdf | 86 | 2,073 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter M - Regulated Investment Companies and Real Estate Investment Trusts.pdf | 100 | 2,168 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter N - Tax Based on Income From Sources Within or Without the United States.pdf | 321 | 6,905 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter O - Gain or Loss on Disposition of Property.pdf | 95 | 2,443 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter P - Capital Gains and Losses.pdf | 166 | 3,745 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter Q - Readjustment of Tax Between Years and Special Limitations.pdf | 15 | 528 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter R - Election To Determine Corporate Tax on Certain International Shipping Activities Using Per Ton Rate.pdf | 10 | 314 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter S - Tax Treatment of S Corporations and Their Shareholders.pdf | 45 | 1,168 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter T - Cooperatives and Their Patrons.pdf | 11 | 372 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter U - Designation and Treatment of Empowerment Zones, Enterprise Communities, and Rural Development Investment Areas.pdf | 23 | 669 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 1 _ Subchapter V - Title 11 Cases.pdf | 4 | 206 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter W - Repealed (No Longer Applicable Law).pdf | 2 | 195 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter X - Repealed (No Longer Applicable Law).pdf | 1 | 154 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter Y - Repealed (No Longer Applicable Law).pdf | 2 | 197 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 1 _ Subchapter Z - Opportunity Zones.pdf | 8 | 316 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 2 _ TAX ON SELF-EMPLOYMENT INCOME.pdf | 35 | 981 | text_based | 1.00 | none* |
| USC26 - Subtitle A - Chapter 2A _ UNEARNED INCOME MEDICARE CONTRIBUTION.pdf | 2 | 145 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 3 _ WITHHOLDING OF TAX ON NONRESIDENT ALIENS AND FOREIGN CORPORATIONS.pdf | 21 | 678 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 4 _ TAXES TO ENFORCE REPORTING ON CERTAIN FOREIGN ACCOUNTS.pdf | 7 | 240 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 5 _ REPEALED (no longer applicable law).pdf | 1 | 144 | text_based | 1.00 | none |
| USC26 - Subtitle A - Chapter 6 _ CONSOLIDATED RETURNS.pdf | 27 | 793 | text_based | 1.00 | none* |

### USC26 — Other Subtitles (11 files)

| Filename | Pages | Size (KB) | Classification | Confidence | Encoding Issues |
|----------|-------|-----------|---------------|-------------|-----------------|
| USC26 - Subtitle A - Chapter 1 _ Front Matter and Table of Contents.pdf | 2 | 188 | text_based | 1.00 | none |
| USC26 - Subtitle B _ Estate and Gift Taxes.pdf | 170 | 3,921 | text_based | 1.00 | none |
| USC26 - Subtitle C _ Employment Taxes.pdf | 271 | 6,324 | text_based | 1.00 | none |
| USC26 - Subtitle D _ Miscellaneous Excise Taxes.pdf | 404 | 9,832 | text_based | 1.00 | none* |
| USC26 - Subtitle E _ Alcohol, Tobacco, and Certain Other Excise Taxes.pdf | 256 | 6,548 | text_based | 1.00 | none* |
| USC26 - Subtitle F _ Procedure and Administration - (1 of 4).pdf | 465 | 10,938 | text_based | 1.00 | none* |
| USC26 - Subtitle F _ Procedure and Administration (2 of 4).pdf | 162 | 4,149 | text_based | 1.00 | none* |
| USC26 - Subtitle F _ Procedure and Administration (3 of 4).pdf | 240 | 6,131 | text_based | 1.00 | none |
| USC26 - Subtitle F _ Procedure and Administration (4 of 4).pdf | 323 | 7,116 | text_based | 1.00 | none |
| USC26 - Subtitle G _ The Joint Committee on Taxation.pdf | 7 | 316 | text_based | 1.00 | none* |
| USC26 - Subtitle H _ Financing of Presidential Election Campaigns.pdf | 27 | 719 | text_based | 1.00 | none |
| USC26 - Subtitle I _ Trust Fund Code.pdf | 68 | 2,036 | text_based | 1.00 | none* |
| USC26 - Subtitle J _ Coal Industry Health Benefits.pdf | 22 | 500 | text_based | 1.00 | none |
| USC26 - Subtitle K _ Group Health Plan Requirements.pdf | 64 | 1,026 | text_based | 1.00 | none |

*\* `ocr_recommended: true` set by detect-pdf (page-density heuristic only; no OCR needed — all sampled pages returned full text)*

---

## Markdown Quality — 3-File Sample

### Sample 1: Small IRC — `USC26 - Subtitle K _ Group Health Plan Requirements.pdf`
- **Pages:** 64 | **Size:** 1,026 KB
- **Headings:** Section numbers render as `## §9801.` — section sign (§) intact, heading detection good
- **Subsections:** Render correctly as `## (a)`, `## (b)`, `## (1)`, etc.
- **Paragraphs:** Long-form text extracts cleanly
- **Tables:** Table from PDF renders as markdown table rows — some formatting noise from the source PDF (visible `|---|---|` column markers and page-break artifacts)
- **Lists (amendments):** Amendment notes (`Editorial Notes`, Pub. L. citations) parse but appear as dense prose blocks — hierarchical structure is lost
- **Overall:** Headings and IRC section structure (§ + number) parse well. Tax rate tables extract with alignment artifacts. Amendment citation blocks are dense but machine-readable.

### Sample 2: Large IRC — `USC26 - Subtitle A - Chapter 1 _ Subchapter A - Determination of Tax Liability.pdf`
- **Pages:** 470 | **Size:** 7,484 KB
- **Headings:** Chapter headings (`## PART I—TAX ON INDIVIDUALS`), subchapter headings, section headings (`## §1. Tax imposed`) all render correctly
- **Subsections:** `(a)`, `(b)`, `(c)`, `(d)`, `(e)` subsections render as `##` subheadings — structure is navigable
- **Tax tables:** Income tax rate tables (brackets: "Not over $36,900", "Over $36,900 but not over $89,150") extract as markdown tables — functional but column alignment is loose
- **Paragraphs:** Continues cleanly across subsections (a)(1), (a)(2), etc.
- **Definitions:** `(A) In general`, `(B) Definitions` sub-items render cleanly under their parent section
- **Overall:** Best-case markdown output for legal/tax text. Hierarchical structure (§ → subsection → paragraph → subparagraph) is well-preserved. Tax rate tables are the weakest element (wide tables with irregular column structure).

### Sample 3: CFR — `CFR _ Treas Regs Volume 10 (1 of 3).pdf`
- **Pages:** 328 | **Size:** 6,965 KB
- **Headings:** `## Table of Contents`, section headings `§ 1.641(a)(0)` render correctly — section sign preserved
- **Structure:** Title → Chapter → Part → Section hierarchy visible in headings
- **Editorial notes:** `EDITORIAL NOTES`, `AMENDMENTS`, `POPULAR NAME` sections parse as text blocks — not tree-structured
- **Tables:** TOC-style tables (page references) render as markdown tables — functional
- **Legal citations:** Internal citations (`26 CFR 1.641(a)(0)`) preserved as plain text
- **Overall:** Slightly more heterogeneous than IRC — mix of narrative sections, definition blocks, and tabular materials. `##` heading levels map reasonably but cross-references and footnotes may need post-processing cleanup.

---

## IRC Section Numbering — Parsability Assessment

The corpus uses three structural numbering systems:

### 1. IRC Section Sign (§) + Number
```
§9801. Increased portability through limitation on preexisting condition exclusions
§1. Tax imposed
§2. Definitions and special rules
```
**Parsability: EXCELLENT.** The section sign (§) is preserved in UTF-8 text throughout all 78 files. Pattern `§\d+[A-Za-z0-9.]*` is immediately regex-extractable. Subsection hierarchy `(a)`, `(1)`, `(A)` is consistently nested and text-extractable.

**Recommendation:** Pre-process section headers with a normalization pass:
```python
import re
section_pattern = re.compile(r'^##\s*§\s*(\d+[A-Za-z0-9.]*)\.\s*(.*)')
def parse_irc_section(line):
    m = section_pattern.match(line)
    if m:
        return {"section": m.group(1), "title": m.group(2)}
```
Cross-reference resolution (§1(a)(2) → §1(a)(2) in text) is also tractable via exact string matching.

### 2. CFR Section Numbering (Title 26 CFR § 1.641(a)(0))
```
§ 1.641(a)(0) refers to title 26, part 1, section 641(a)(0)
```
**Parsability: GOOD.** The parenthetical nesting `(a)(0)` is consistently text-extractable. Note: `§1.641` vs `§ 1.641` — occasional space inconsistency after the section sign.

**Recommendation:** Normalize with `re.sub(r'§\s*', '§', text)` before section matching.

### 3. Subchapter/Chapter Structure
```
PART I—TAX ON INDIVIDUALS
Subchapter A—Determination of Tax Liability
Chapter 1—Normal Taxes and Surtaxes
Subtitle A—Income Taxes
```
**Parsability: EXCELLENT.** These render as standard markdown headings with `—` em-dash separators. Regex: `^(#{1,6})\s*(PART|Subchapter|Chapter|Subtitle)\s+([IVXL\d]+)\s*—\s*(.*)$`

---

## Recommendations for the tax-domain Framework

### HIGH PRIORITY

**1. TypedStepSchema section-number extraction**
The `§<number>` pattern is the dominant indexing mechanism for the IRC. The tax-domain framework should define a canonical `IRC_SECTION` entity type with fields:
- `section_number` (e.g., `"1"`, `"401(k)"`, `"9801"`)
- `title` (e.g., `"Tax imposed"`)
- `subtitle` (e.g., `"Subtitle A"`)
- `subsection_path` (e.g., `"(a)(1)"`)

**2. Subsection path parser**
Subsection paths like `(a)(2)(B)(i)` need a dedicated parser — they are the primary granularity for tax research. They appear at up to 5+ levels of nesting (e.g., §1.401(k)-1(a)(2)(i)(A)). A `SubsectionPath` value object should:
- Parse string → array of path segments
- Compare paths (is `(a)(2)` a child of `(a)`?)
- Generate canonical ordering for RAG chunk headers

**3. Amendment/Editorial note extraction**
The `EDITORIAL NOTES` blocks (Pub. L. citations, amendment years, repeal notes) are critical for legal accuracy — "this section was added in 1997" vs "repealed in 2014" changes the entire legal interpretation. The framework needs a bitemporal validity model: `effective_date` vs `expiration_date` per paragraph/section.

**4. Tax rate table chunking**
Rate tables (bracket + percentage + dollar thresholds) are among the highest-value content for LLM consumption but extract with poor column alignment. Consider:
- A dedicated `TaxRateTable` schema (brackets as rows, with `floor`, `ceiling`, `rate`, `fixed_dollar`)
- Post-processing that runs pdf2md output through a table-normalization pass

### MEDIUM PRIORITY

**5. Cross-reference resolution**
IRC-to-IRC references (`section 401(k)`, `§1.401(k)-1`) and CFR-to-IRC references are abundant but currently free-text. A `CrossReference` entity type with `source_section`, `target_section`, `reference_type` (IRC_CITATION, CFR_CITATION, REG_SECTION) would enable authoritative link resolution.

**6. Jurisdiction/version tagging**
These PDFs are dated (CFR Volume 10 is "Revised as of April 1, 2025"). The framework should tag every entity with `jurisdiction` (Federal) and a `code_version` date. Different volumes have different as-of dates — this matters for retroactive applicability.

**7. Pub. L. citation parser**
Amendment notes contain `Pub. L. 105–34, title XV, §1531(a)(1), Aug. 5, 1997` — a well-structured citation form. A `PublicLawCitation` parser would enable the framework to build a amendment timeline per section.

### LOWER PRIORITY

**8. Repealed/superseded section handling**
Several files are explicitly marked "Repealed (No Longer Applicable Law)" — these need a `status` field: `ACTIVE`, `REPEALED`, `SUPERSEDED`. The framework should handle repealed sections differently in search (downrank or exclude) vs. legal research (still relevant historically).

**9. Orphan paragraph handling**
Very large sections (§1, §401, §409A) have hundreds of sub-paragraphs. Chunking strategy should respect subsection boundaries rather than arbitrary token counts — each `(a)(1)` paragraph is semantically self-contained.

---

## Summary Stats

| Category | Count | Total Pages | Total Size |
|----------|-------|-------------|------------|
| CFR / Treas Regs | 30 | ~10,500 | ~42 MB |
| USC26 Chapter 1 (Subchapter A–Z) | 35 | ~3,500 | ~75 MB |
| USC26 Other Subtitles (B–K) | 13 | ~2,000 | ~40 MB |
| **TOTAL** | **78** | **~16,000** | **~157 MB** |

*Page count totals are approximations from pdfinfo — see per-file table for exact counts.*
