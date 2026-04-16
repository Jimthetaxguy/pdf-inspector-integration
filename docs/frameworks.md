# PDF capability frameworks — detailed design

<!-- STATUS: SCAFFOLD — frameworks defined, not yet implemented -->

Companion docs:
- [`SPEC.md`](SPEC.md) — capabilities, architecture, sourcing
- [`assessment.md`](assessment.md) — Part 1 baseline + API surface
- `~/.claude/plans/agile-juggling-waffle.md` — execution plan

> **Shareability.** Safe to share after stripping

### Purpose

End-to-end processing of tax-related PDFs: classify, extract structured
fields, assemble filing packets, redact PII, and generate audit
workpapers. Connects pdf-inspector's extraction primitives to the
`tax-domain` skill's entity model.

### Trigger

- "Analyze this tax form"
- "Extract W-2 / 1099 / K-1 fields"
- "Assemble tax return packet"
- "Redact SSN from this PDF"
- Any PDF arriving in a tax-workspace context

### Flow

```
Tax PDF(s) arrive
  │
  ▼
┌─────────────────────────────────────────────────────────┐
│  F2.1  Triage (calls F1)                                │
│  batch_classify → SHA-256 dedup → route text vs scanned │
└──────────┬──────────────────────────────────────────────┘
           │
           ▼ (text-based PDFs)
┌─────────────────────────────────────────────────────────┐
│  F2.2  Form identification                              │
│  Detect form type from page-1 text patterns:            │
│    • "Form W-2" → w2 template                           │
│    • "Form 1099-INT" → 1099_int template                │
│    • "Schedule K-1 (Form 1065)" → k1_1065 template      │
│    • "U.S. Individual Income Tax Return" → 1040          │
│    • Unrecognized → generic markdown extraction          │
│                                                         │
│  Implementation: regex over first 500 TextItems from    │
│  extract_text_with_positions. No ML, no OCR.            │
└──────────┬──────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────┐
│  F2.3  Anchor-based field extraction                    │
│                                                         │
│  For each recognized form type:                         │
│  1. Locate anchor labels in TextItem stream             │
│     (e.g., "Wages, tips, other compensation")           │
│  2. Compute extraction rect relative to anchor position │
│     (below, right-of, or same-line-after)               │
│  3. Call extract_text_regions with computed rects        │
│  4. Parse extracted text into typed fields               │
│     (amounts → f64, dates → NaiveDate, SSN → masked)    │
│                                                         │
│  This is robust to layout variance across issuers       │
│  because anchors are semantic, not positional.          │
└──────────┬──────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────┐
│  F2.4  TypedStepSchema mapping                          │
│  Map extracted fields → tax-domain entity model:        │
│    • W2Fields { wages, fed_tax_withheld, ss_wages, ... }│
│    • K1Fields { ordinary_income, rental_income, ... }   │
│    • Form1099IntFields { interest_income, ... }         │
│  Feed into tax-domain skill for:                        │
│    • Jurisdiction rule application                      │
│    • Multi-year comparison                              │
│    • Bitemporality tracking (as-filed vs as-amended)    │
└──────────┬──────────────────────────────────────────────┘
           │
           ▼ (optional write-side paths)
┌─────────────────────────────────────────────────────────┐
│  F2.5  Assembly + redaction (requires lopdf write-side) │
│  • merge_pdfs: combine 1040 + schedules + K-1s + W-2s  │
│    into single filing packet with bookmarks             │
│  • redact_pdf: strip SSN/account/DoB patterns           │
│  • Workpaper generation: structured PDF with provenance │
│    citations linking extracted values → source pages     │
└─────────────────────────────────────────────────────────┘
```

### MCP tools (new, Part 3)

| Tool | Input | Output | Priority |
|---|---|---|---|
| `identify_tax_form` | `{path}` | `{form_type, confidence, variant}` | P1 |
| `extract_tax_fields` | `{path, form_type?}` | `{form_type, fields: {name→value}}` | P1 |
| `assemble_tax_packet` | `{paths[], out, bookmarks?}` | `{out, page_count}` | P3 (write-side) |

### Anchor template registry

Each form type has an anchor template — a list of `(label_regex,
spatial_relation, field_name, value_parser)` tuples.

Example for W-2:
```
[
  ("Wages,? tips,? other comp", RightOf, "wages", parse_currency),
  ("Federal income tax withheld", RightOf, "fed_tax_withheld", parse_currency),
  ("Social security wages", RightOf, "ss_wages", parse_currency),
  ("Social security tax withheld", RightOf, "ss_tax_withheld", parse_currency),
  ("Employer.s name", Below, "employer_name", parse_text),
  ("Employee.s (first |)name", Below, "employee_name", parse_text),
]
```

Templates live in `crates/pdf-inspector-skillkit/src/domain/tax/templates/`
as Rust const arrays. Adding a new form = adding a new template file.

### Existing skills composed

| Skill | How it's used |
|---|---|
| `pdf-routing` (F1) | Classify + route at entry |
| `tax-domain` | TypedStepSchema entity model, jurisdiction rules |
| `rag-vector-search` | Index extracted fields for multi-year queries |
| `pdf-viewer` | Interactive review of extraction results |
| `verification-before-completion` | End-gate |

### Routing chain

```
"Analyze tax return" / "Extract W-2 fields" / "Tax form shoebox"
  → pdf-routing (F1) classify
  → tax-document:identify_tax_form
  → tax-document:extract_tax_fields
  → tax-domain (TypedStepSchema mapping)
  → verification-before-completion
```

### Skill spec

- **Name:** `tax-document`
- **Domain:** `integration`
- **Category:** `document-processing`
- **Requires:** `pdf-routing`, `tax-domain`
- **Triggers:** `tax form`, `W-2 extract`, `1099 extract`, `K-1 extract`,
  `tax shoebox`, `extract tax fields`, `analyze tax return`
- **Maturity target:** Silver (Part 3 P2 ship), Gold after 30 days
- **Platforms:** all 5

### Graduation gate

- [ ] Correctly identifies ≥90% of form types across 5+ issuers per form
- [ ] Extracts key financial fields (wages, interest, ordinary income)
      with ≤2% error rate on typed parsing
- [ ] Handles multi-page K-1s (common: 3-5 pages) without field confusion
- [ ] Redaction strips all SSN/EIN patterns (tested with regex + manual review)
- [ ] TypedStepSchema round-trip: extract → serialize → deserialize → compare
- [ ] Response time: identify <100ms, extract <500ms per form

---

## F3 — SEC Filing Framework (HIGH priority)

### Purpose

Bulk ingestion, section-splitting, and financial table extraction from
SEC EDGAR filings (10-K, 10-Q, 8-K). Connects pdf-inspector to the
existing `sec-extraction-toolkit` pipeline, replacing the current
all-OCR path with a classify-first approach that's 5–50× faster on
text-based filings (~96% of EDGAR PDFs are text-based).

### Trigger

- "Analyze this 10-K"
- "Extract financial tables from SEC filing"
- "Bulk ingest EDGAR filings"
- "Split this filing into sections"

### Flow

```
SEC filing PDF(s) arrive (from sec-extraction-toolkit EDGAR download)
  │
  ▼
┌──────────────────────────────────────────────────┐
│  F3.1  Triage (calls F1)                         │
│  batch_classify → expect ~96% TextBased          │
│  Route the ~4% scanned to OCR tier               │
└──────────┬───────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────┐
│  F3.2  Section splitting                         │
│  Regex-driven markdown post-processor:           │
│    • "ITEM 1\." → item_1_business                │
│    • "ITEM 1A\." → item_1a_risk_factors          │
│    • "ITEM 7\." → item_7_mda                     │
│    • "ITEM 8\." → item_8_financial_statements    │
│  Uses heading hierarchy from pdf_to_markdown     │
│  to locate section boundaries.                   │
│                                                  │
│  Output: Vec<Section { name, start_page,         │
│           end_page, markdown }>                   │
└──────────┬───────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────┐
│  F3.3  Financial table normalization             │
│  Post-process tables from analyze_layout:        │
│    • Detect subtotal/total rows (bold, indent)   │
│    • Parse bracketed negatives: (1,234) → -1234  │
│    • Handle continuation tables across pages     │
│    • Align "in millions" / "in thousands" units   │
│    • Strip footnote markers (¹ ² ³ etc.)         │
│                                                  │
│  Output: Vec<FinancialTable { name, period,      │
│           rows: Vec<Row { label, values[] }> }>  │
└──────────┬───────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────┐
│  F3.4  RAG index + structured output             │
│  Per section and table:                          │
│    • Embed into rag-vector-search                │
│    • Store structured JSON in QMD collection     │
│    • Generate comparison tables across filings   │
└──────────────────────────────────────────────────┘
```

### MCP tools (new, Part 3)

| Tool | Input | Output | Priority |
|---|---|---|---|
| `split_sec_filing` | `{path}` | `{sections[{name, page_range, markdown}]}` | P2 |
| `normalize_financial_tables` | `{path, section?}` | `{tables[{name, period, rows[]}]}` | P3 |

### Existing skills composed

| Skill | How it's used |
|---|---|
| `pdf-routing` (F1) | Classify + extract |
| `sec-extraction-toolkit` | EDGAR download, XBRL fallback, ratio analysis |
| `rag-vector-search` | Embed sections for semantic retrieval |
| `python-ai-patterns` | Data pipeline for table→DataFrame conversion |
| `enterprise-ai-platform` | Multi-tenant storage if serving multiple analysts |

### Routing chain

```
"Analyze SEC filing" / "Extract 10-K tables" / "Ingest EDGAR"
  → sec-extraction-toolkit (download if URL)
  → pdf-routing (F1) classify
  → sec-filing:split_sec_filing
  → sec-filing:normalize_financial_tables
  → rag-vector-search (embed)
  → verification-before-completion
```

### Skill spec

- **Name:** `sec-filing`
- **Domain:** `integration`
- **Category:** `financial-data`
- **Requires:** `pdf-routing`, `sec-extraction-toolkit`
- **Triggers:** `sec filing`, `10-k extract`, `financial tables`,
  `edgar ingest`, `split filing sections`, `md&a`, `risk factors`
- **Maturity target:** Silver
- **Platforms:** all 5

### Graduation gate

- [ ] Section split accuracy ≥90% on 20 10-K filings from different
      companies/years
- [ ] Financial table normalization handles bracketed negatives, unit
      labels, continuation rows across ≥10 real income statements
- [ ] End-to-end: EDGAR URL → searchable RAG index in <30s per filing
- [ ] No data loss: every section of the filing is captured (even
      "exhibits" and "signatures" catch-all)

---


---

## Cross-framework composition matrix

Shows how the five frameworks share primitives and how data flows
between them:

| Primitive | F1 (Routing) | F2 (Tax) | F3 (SEC) | F4 (Career) | F5 (Archive) |
|---|---|---|---|---|---|
| `classify_pdf` | ★ core | calls F1 | calls F1 | calls F1 | calls F1 |
| `pdf_to_markdown` | ★ core | for unrecognized forms | full text | full text | receipt text |
| `analyze_layout` | on request | table detection | ★ tables core | — | — |
| `extract_text_regions` | — | ★ anchor fields | — | — | — |
| `extract_table_regions` | — | schedules | ★ financials | — | — |
| `batch_classify` | folder triage | shoebox | EDGAR bulk | — | folder |
| Form identification | — | ★ core | section split | doc-type detect | vendor detect |
| Anchor extraction | — | ★ core | — | — | — |
| Table normalization | — | — | ★ core | — | — |
| Redaction (write-side) | — | SSN/EIN strip | — | PII strip | — |
| Merge (write-side) | — | packet assembly | — | — | — |
| `tax-domain` | — | ★ consumer | — | — | — |
| `sec-extraction-toolkit` | — | — | ★ consumer | — | — |
| `career-pipeline` | — | — | — | ★ consumer | — |
| `rag-vector-search` | — | multi-year | ★ embed | — | — |
| `pdf-viewer` | display | review | review | — | — |

**★ = primary use of that primitive within the framework**

---

## Implementation priority & sequencing

| Phase | Framework | What ships | Depends on | Target |
|---|---|---|---|---|
| **Part 2** (now) | F1 | classify + markdown MCP tools, SKILL.md, routing chain | pdf-inspector upstream | Week of 2026-04-15 |
| **Part 2.3b** | F1 | analyze_layout, extract_*_regions, batch_classify | Part 2.3a skeleton | Week of 2026-04-21 |
| **Part 3 P1** | F1 | scanned-heuristics post-processor (smart OCR routing) | Part 2 complete | Week of 2026-04-28 |
| **Part 3 P2** | F2 | form identification + anchor extraction (W-2, 1099, K-1) | F1 complete + tax-domain | Week of 2026-05-05 |
| **Part 3 P3** | F3 | section splitter + financial table normalizer | F1 complete + sec-toolkit | Week of 2026-05-12 |
| **Part 3 P4** | F4 | offer/contract/JD extraction | F1 complete + career-pipeline | Week of 2026-05-19 |
| **Part 3 P5** | F2 | write-side: merge + redact (requires lopdf decision §6.2) | F2 P2 complete | Week of 2026-05-26 |
| **Ongoing** | F5 | personal archive (Draft tier, low effort) | F1 complete | opportunistic |

### Energy alignment


---

## Quality tier targets per framework

| Framework | Ship tier | 30-day target | Rationale |
|---|---|---|---|
| F1 (Routing) | Silver | Gold | Infrastructure must be reliable; Gold requires perf benchmarks + decision trees |
| F2 (Tax) | Silver | Gold | Highest domain value; Gold after multi-issuer testing |
| F3 (SEC) | Bronze | Silver | Bulk-oriented; Silver after 20-filing evaluation |
| F4 (Career) | Bronze | Silver | Active pipeline use validates quickly |
| F5 (Archive) | Draft | Draft | Personal use; never graduates past Draft |

### What each tier requires (from skill-library-framework v3.1)

| Tier | Key requirements |
|---|---|
| Draft | SKILL.md exists, basic instructions, listed in master-index |
| Bronze | 3+ triggers, platforms listed, examples, correct frontmatter |
| Silver | references/ dir, error guidance, output conventions, library chooser |
| Gold | Performance benchmarks, decision trees, 3+ examples, quality standards |
| Diamond | Automated monitoring, cross-platform consistency verification, regression tests |

---

## Verification strategy

### Per-framework test corpus

| Framework | Corpus | Source | Size |
|---|---|---|---|
| F1 | Stratified: 10 text + 10 scanned + 10 tax-form + 10 mixed | local docs + EDGAR + sample scans | 40 PDFs |
| F2 | W-2 (5 issuers) + 1099-INT (3) + 1099-DIV (2) + K-1 (5) + 1040 (3) | prior-year returns | 18 forms |
| F3 | 10-K filings from 10 S&P 500 companies, 2 years each | EDGAR | 20 filings |
| F4 | Offer letters (5) + contracts (3) + JDs (5) | career pipeline | 13 docs |
| F5 | Receipts (10) + bills (5) | household | 15 docs |

### End-to-end integration tests

1. **Tax pipeline test:** W-2 PDF → F1 classify → F2 identify → F2 extract
   → tax-domain TypedStepSchema → serialized JSON → round-trip compare.
   Must complete in <2s.

2. **SEC pipeline test:** EDGAR 10-K URL → sec-toolkit download → F1
   classify → F3 split → F3 normalize tables → rag-vector-search embed.
   Must complete in <30s.

3. **Career pipeline test:** Offer letter PDF → F1 classify → F4 extract
   → career-pipeline pipeline tracker entry. Must complete in <5s.

4. **Cross-framework routing test:** Drop 10 mixed PDFs (tax + SEC +
   career + random) into a folder. `batch_classify` → each routes to
   the correct framework based on content. Zero misroutes.

---

## Open design decisions

1. **Anchor extraction: pure regex vs LLM-assisted?**
   Pure regex is fast (<10ms) and deterministic, but may miss variant
   layouts. LLM-assisted (send TextItems + ask "where is the wages
   field?") is flexible but adds latency + token cost. **Lean: regex
   first, LLM fallback on extraction failure.** Test on the 18-form
   tax corpus to measure regex-only accuracy before adding LLM.

2. **Financial table normalization: Rust or Python?**
   Rust keeps it in-process (fast, no FFI). Python gives access to
   pandas/polars for DataFrame operations. **Lean: Rust for parsing
   (bracketed negatives, unit labels), export to JSON for downstream
   Python/pandas consumers.** The `sec-extraction-toolkit` is already
   Python-based.

3. **Where do anchor templates live?**
   Options: (a) hardcoded Rust const arrays in skillkit, (b) JSON files
   loaded at runtime, (c) QMD collection. **Lean: Rust const arrays
   for core forms (W-2, 1099, K-1), JSON sidecar files for user-added
   templates.** Rust arrays compile into the binary (no runtime I/O);
   JSON allows extension without rebuild.

4. **Should F3 section splitter use heading hierarchy or regex?**
   10-K Item headings are standardized but inconsistently formatted
   (some bold, some ALL CAPS, some indented). **Lean: regex on
   markdown text (post-heading-detection), not on raw TextItems.**
   pdf-inspector already normalizes headings via font-size heuristics;
   work from that output.

5. **Cross-framework routing: explicit dispatch or auto-detect?**
   When `batch_classify` runs on a folder, should each PDF auto-route
   to F2/F3/F4/F5 based on content, or should the user specify?
   **Lean: auto-detect with confirmation.** Show the routing plan,
   let user approve or override, then execute.
