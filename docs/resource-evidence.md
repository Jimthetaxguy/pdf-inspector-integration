# AnyDoc worker resource evidence

**Observed:** 2026-08-28
**Reference host:** Darwin/arm64
**Build profile:** `cargo test --release`
**Provider:** AnyDoc 0.2.4
**Scope:** One public, non-PII positive fixture per enabled non-PDF lane

The evaluator is `worker_resource_evidence_covers_enabled_lanes` in
`crates/pdf-inspector-mcp/tests/document_tool.rs`. It launches the real
workspace worker with `ANYDOC_RESOURCE_EVIDENCE=1`, sends the same framed
worker protocol used by the production adapter, and records:

- `peak_rss_bytes`: the worker's `getrusage(RUSAGE_SELF).ru_maxrss`, normalized
  to bytes on Unix (Linux reports KiB and Darwin reports bytes).
- `response_bytes`: the complete framed worker response, including its 16-byte
  protocol header.
- `wall_clock_ms`: monotonic elapsed time from worker spawn through exit.

## Release observations

| Lane | Fixture | Peak RSS (bytes) | Response bytes | Wall time (ms) |
|---|---|---:|---:|---:|
| DOCX | `docx/public-fixture.docx` | 8,617,984 | 171 | 11 |
| PPTX | `pptx/public-walkthrough.pptx` | 8,896,512 | 274 | 3 |
| XLSX | `xlsx/public-workpaper.xlsx` | 8,749,056 | 320 | 2 |
| ODS | `ods/public-workpaper.ods` | 9,175,040 | 496 | 3 |
| ODT | `odt/minimal.odt` | 8,454,144 | 103 | 2 |
| ODP | `odp/public-presentation.odp` | 9,863,168 | 374 | 3 |
| EPUB | `epub/public-spine-order.epub` | 8,749,056 | 565 | 2 |

These are reproducible observations for the recorded host/profile, not
cross-platform performance guarantees. The evaluator asserts that every
worker returns a positive response, stays below the existing 8 MiB public
output envelope, and completes before the existing 15-second worker deadline.
The opt-in metadata is omitted from normal worker responses.

## Boundary and interpretation

The existing worker still enforces an 8 MiB Markdown cap and a 15-second
deadline. Linux additionally applies a 1 GiB `RLIMIT_AS` and a seccomp
network-denial filter; Darwin launches the worker under macOS named
`no-network` profile. Darwin and other non-Linux hosts do not yet have an
equivalent production memory ceiling. Therefore this evidence:

- supports the current enabled-lane baseline and can detect gross release
  regressions;
- does not establish a hostile-input peak-memory budget;
- proves only the scoped worker-level no-network canaries on the platforms
  where they run;
- does not establish filesystem isolation or non-Linux process/memory
  containment;
- does not justify broadening the strict ODP or EPUB routes beyond their Linux memory-gated contracts, or enabling RTF, legacy, macro-enabled, CSV on non-Linux hosts, or source-coordinate behavior;

Run the release observation with:

```text
cargo test --release -p pdf-inspector-mcp --test document_tool worker_resource_evidence_covers_enabled_lanes --locked -- --exact --nocapture
```

## Upstream AnyDoc abuse observations

The evaluator `scripts/evaluate-upstream-abuse.py` runs the reviewed abuse
fixtures from a caller-supplied sibling `firecrawl-anydoc` mirror. The fixtures
are not copied into this repository and the evaluator emits only logical names,
stable result codes, and bounded measurements.

**Observed:** 2026-08-28 on Darwin/arm64, release worker, macOS `no-network`
profile.

| Fixture | Input bytes | Expected/actual | Peak RSS (bytes) | Wall time (ms) |
|---|---:|---|---:|---:|
| `deepxml--errors.docx` | 1,257 | `resource_limit` / `resource_limit` | 12,746,752 | 7 |
| `imagebomb--errors.docx` | 197,082 | `resource_limit` / `resource_limit` | 8,110,080 | 5 |
| `zipbomb--errors.docx` | 196,603 | `resource_limit` / `resource_limit` | 8,060,928 | 5 |
| `hugespan--errors.pptx` | 1,749 | `resource_limit` / `resource_limit` | 8,601,600 | 4 |
| `emptyrowrepeat--errors.ods` | 466 | `resource_limit` / `resource_limit` | 8,273,920 | 4 |
| `hugespan--errors.ods` | 465 | `resource_limit` / `resource_limit` | 8,355,840 | 4 |
| `hugerepeat--errors.ods` | 483 | `resource_limit` / `resource_limit` | 8,355,840 | 4 |

The run passed 7/7 cases with zero nonzero worker exits, protocol failures,
raw stderr, or timeout outcomes. These are initial Darwin hostile-resource
observations, not a universal memory budget or cross-platform proof; repeat the
evaluator after upstream revision changes and on Linux CI before promotion.

Run it with a local mirror and release worker:

```text
python3 scripts/evaluate-upstream-abuse.py \
  --mirror /path/to/firecrawl-anydoc \
  --worker target/release/pdf-inspector-mcp
```

The instrumentation is internal and opt-in. It adds no public document fields,
no upstream source, no dependency, and no change to the default MCP response
contract.
