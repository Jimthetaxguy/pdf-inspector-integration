# Dependency PR review — 2026-08-22

Scope: live review of public pull requests
[#14](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/14) through
[#18](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/18), plus a combined
lockfile validation and transitive RustSec scan.

The literal `svg` strings in the copied PR list are GitHub status-icon copy
artifacts. They are not pull requests or repository files.

## Live state

All five PRs were open, clean, mergeable, one commit ahead of `main`, and zero
commits behind when inspected. Each changes only `Cargo.lock`.

| PR | Head | Lockfile change | Compatibility score | Decision |
|---|---|---|---:|---|
| [#18 — anyhow 1.0.103 → 1.0.104](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/18) | `a82ec65067ad05ba595b8fb2140049a08af0991f` | Version and checksum | 89% | Safe maintenance update |
| [#14 — serde_json 1.0.150 → 1.0.151](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/14) | `02ed6dbaf63b669f8268e6b5fbe7ac2b7c9aa48e` | Version and checksum | 78% | Safe maintenance update |
| [#16 — thiserror 2.0.18 → 2.0.19](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/16) | `ce005bd2ec557c8b8279a45c42cd982207ff1a12` | Adds `syn 3`; +34/-23 | 76% | Supersede with current 2.0.20 |
| [#15 — regex 1.12.4 → 1.13.1](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/15) | `8f1c8247cf08f82fb6fdfeebc24337cd12a7f40b` | Also updates `regex-automata` | 79% | Accept with parser and resource regression coverage |
| [#17 — tokio 1.52.3 → 1.53.1](https://github.com/Jimthetaxguy/anydoc-enhanced/pull/17) | `30af694e0d8f7c900518677504ed67543e5b7c23` | Version and checksum | 80% | Accept with timeout and stdio smoke coverage |

## Corrections to the initial QA snapshot

### Four green checks are two jobs run twice

The workflow ran the same `test` and `cargo-deny` jobs for both `push` and
`pull_request`. The test job itself covers formatting, Clippy, workspace tests,
and a release build. The four badges do not represent four independent job
categories.

This branch limits `push` CI to `main`, leaving one pull-request run for feature
branches and one post-merge run on `main`.

### The advisory job did not check advisories

The job was named `cargo-deny (licenses + advisories)` but executed only:

```text
cargo deny check licenses bans
```

A live transitive scan found
[`RUSTSEC-2026-0204`](https://rustsec.org/advisories/RUSTSEC-2026-0204)
in `crossbeam-epoch 0.9.18`. The dependency path is:

```text
pdf-inspector / lopdf → rayon → rayon-core → crossbeam-deque → crossbeam-epoch
```

Updating `crossbeam-epoch` to `0.9.20` removes the known vulnerability. The
scan still reports
[`RUSTSEC-2026-0192`](https://rustsec.org/advisories/RUSTSEC-2026-0192), an
unmaintained warning for `ttf-parser 0.25.1`. The current AnyDoc/PDF candidate
stack also resolves that version, so the parser-convergence gate must resolve
it or record a bounded, owned exception with an upstream trigger; convergence
alone does not remove the warning.

The candidate workflow now runs all four configured cargo-deny checks:

```text
cargo deny --all-features --locked check advisories licenses bans sources
```

### PR #16 is already stale

`thiserror 2.0.20` has superseded `2.0.19`. Version 2.0.19 changes the
proc-macro parser to `syn 3`, so it is not merely a two-line checksum refresh.
The current update is still low risk, but merging 2.0.19 would immediately
create another maintenance step. This branch uses 2.0.20 instead.

### Regex 1.13.1 is a correctness fix

Regex 1.13.1 fixes incorrect leftmost-first match offsets in reverse
suffix/inner optimizations. The domain parsers use captures and match offsets,
so the update is relevant to correctness, not merely dependency freshness.
The combined lockfile resolves `regex-automata 0.4.18`, not the 0.4.16 present
in PR #15. Version 0.4.17 changed its cache pool from a fixed eight stacks to a
size derived from `available_parallelism()`, so high-core hosts can use more
memory. Accept that transitive change only with parser and resource validation.

The new public-fixture integration tests cover the generic PDF pipeline. They
do not invoke the tax, IRC, or SEC parsers and therefore do not close the domain
regression gap. A richer positive IRC/SEC/tax corpus remains an AnyDoc-plan
gate.

## Combined-state validation

A clean disposable checkout was updated to the five requested target versions
and passed:

```text
cargo fmt --all -- --check
cargo metadata --locked --format-version 1
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo audit
cargo deny --locked check advisories licenses bans sources
```

Observed test result before this branch's fixture-test improvement: 28 passed,
0 failed, 4 ignored. The ignored tests were the home-directory-dependent PDF
tests that this branch replaces with tracked public fixtures.

The final local candidate on 2026-08-23 passed formatting, locked metadata,
workspace check, Clippy with warnings denied, 33 tests with zero ignored, the
release build, and all four cargo-deny policies. `cargo audit` found no known
vulnerabilities and retained the documented `ttf-parser 0.25.1` unmaintained
warning.

An MCP stdio smoke initialized the release binary, verified the exact 13-tool
set, called all three public PDF fixtures (4, 35, and 27 pages), and confirmed
that missing-path and unknown-package inputs did not appear in stdout or
stderr—even when raw dependency tracing was requested. This is local evidence;
the changed workflow has no remote CI result until the branch is pushed.

## Recommended execution

Prefer this consolidated baseline over merging the five stale lockfile PRs
one-by-one. It includes the four current target versions, supersedes #16 with
`thiserror 2.0.20`, fixes the transitive RustSec finding, and makes the advisory
gate truthful. After this baseline lands, #14–#18 can be closed as superseded.

If the original PRs must be used instead, the safe dependency order is:

1. #18 `anyhow`
2. #14 `serde_json`
3. refreshed #16 targeting `thiserror 2.0.20`
4. #15 `regex`
5. #17 `tokio`

Rebase each remaining branch after every merge and require checks on its new
head. Do not treat the July 23 check runs as evidence for a rebased lockfile.
