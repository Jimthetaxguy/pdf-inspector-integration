

## Adversarial derivatives

The following synthetic derivatives exercise the strict local boundary through
the production MCP worker:

- `missing-slide.pptx` removes the declared slide part and must return
  `incomplete_conversion`.
- `active-content.pptx` adds an embedded OLE marker and must return
  `active_content_disabled`.

Both are derived from the public inheritance fixture and contain no identifying
metadata or executable payload. Their hashes are recorded in the corpus index.
