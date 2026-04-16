# Contributing to pdf-inspector-integration

Thanks for your interest. This project is small and pragmatic; contributions
that improve real-PDF coverage, fix domain-parser edge cases, or extend the
MCP surface are all welcome.

## Quick start

```bash
git clone https://github.com/Jimthetaxguy/pdf-inspector-integration
cd pdf-inspector-integration
cargo build --workspace
cargo test --workspace
```

## Validate against your own PDFs

The validation runner exercises the domain tools (tax / irc / sec) against
real input files:

```bash
cargo run --example validate_domain -- tax /path/to/your.pdf
cargo run --example validate_domain -- irc /path/to/title-26.pdf
cargo run --example validate_domain -- sec /path/to/10-K.pdf
```

## Code standards

- Format: `cargo fmt --all`
- Lint:   `cargo clippy --workspace --all-targets -- -D warnings`
- Test:   `cargo test --workspace`

All three must pass before a PR is merged.

## PR checklist

- [ ] Tests pass (`cargo test --workspace`)
- [ ] Clippy clean (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] No PII or personal data in test fixtures, commit messages, or logs
- [ ] CHANGELOG.md updated under `[Unreleased]` if the change is user-facing

## Test fixtures

Never commit a real personal or financial PDF — no W-2s, 1099s, K-1s, bank
statements, offer letters, or anything with a name, SSN, account number, or
dollar amount tied to a real person. Use synthetic fixtures (handcrafted PDFs
or anonymized exports) or redact every identifying field before committing.

When in doubt, leave the fixture out and reference it via a path in the test
comment instead.
