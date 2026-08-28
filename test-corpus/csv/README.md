# Public CSV fixture

public-bank-export.csv is a deterministic, synthetic, non-PII bank-export
sample created for the local strict CSV adapter. It contains no real account
numbers, names, addresses, credentials, or client records.

The expected contract is strict UTF-8 (with an optional BOM), RFC-4180-style
quoting, deterministic delimiter sniffing, equal-width rows, bounded fields,
and Markdown escaping. Malformed quoting, ragged rows, oversized fields, and
oversized documents must fail closed. The fixture is not copied from AnyDoc;
the upstream CSV implementation was reviewed for behavior and resource risk.

SHA-256 is recorded in the parent corpus README. The MCP lane is enabled only
on hosts where the worker address-space ceiling is enforceable (currently
Linux). Classification remains available on all platforms.
