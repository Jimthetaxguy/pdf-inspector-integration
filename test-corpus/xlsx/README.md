# Public XLSX fixture

`public-workpaper.xlsx` is a hand-authored, non-PII SpreadsheetML package used
by the AnyDoc XLSX integration test. It contains two visible sheets (`Inputs`
and `Summary`), text values, numeric values, and one formula with a cached
value. It contains no macros, external relationships, hidden content, or
personal metadata.

The package is assembled from the XML sources in `source/` with the system
`zip` utility. The expected behavior is exact `.xlsx` classification,
cached-value-only extraction, stable sheet order, and no formula evaluation.

## Adversarial derivatives

`external-link.xlsx` is derived from the synthetic workbook with an external
link part. The strict route must return `incomplete_conversion` before AnyDoc
conversion and must not fetch the target. Its SHA-256 is recorded in the corpus
index.
