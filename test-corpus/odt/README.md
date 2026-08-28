# ODT fixture corpus

These fixtures are synthetic, public, deterministic inputs for the strict ODT
worker lane. They contain no copied upstream documents, personal information,
machine paths, network payloads, or credentials.

The parser source is the external firecrawl/anydoc dependency at version
0.2.4; the local contract and tests are maintained in this repository.
Upstream reference: https://github.com/firecrawl/anydoc/tree/42bf1c5ecdde9eb0d96d6bd75a9e6698cf93b14c

- public-research-memo.odt: omission regression fixture with visible text,
  list, table, internal anchor, and a `text:note` footnote. AnyDoc 0.2.4
  omits the note body, so the strict lane must return `incomplete_conversion`.
- minimal.odt: positive extraction fixture and smallest valid visible-text package.
- hidden-or-tracked.odt: hidden text and tracked-change markers; must reject.
- external-reference.odt: external hyperlink; must reject as incomplete.
- active-content.odt: form content; must reject as active content.
- missing-image.odt: internal image reference with no archive asset; must reject.
- malformed-content.odt: malformed content.xml; must reject.
- encrypted.odt: manifest encryption marker; must reject.
- wrong-mimetype.odt: spreadsheet mimetype with text content; must reject.
- missing-content.odt: no content.xml; must reject.

The fixture hashes are recorded in test-corpus/README.md.
