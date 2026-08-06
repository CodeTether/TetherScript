## Fixed

**Parser: `&` no longer silently splits a statement.**

`let a = 12 & 10;` used to parse as two statements — `let a = 12`, then a
discarded `&10` borrow — so it printed `12` and exited `0`. Anyone arriving from
C, Python, or Rust's integer types reads that as bitwise AND and gets a wrong
answer with no diagnostic, which is worse than a hard error.

tetherscript has no bitwise AND, so `&` between two expressions is never valid.
It is now a named parse error on both backends and in `check`:

```
$ tetherscript run bitwise.tether
tetherscript: bitwise.tether:1:12: parse error: `&` is not a binary operator;
tetherscript has no bitwise AND. Use `&&` for logical and, or write `&value`
as a prefix to borrow.
```

Prefix `&x` and `&mut x` are unchanged. All 103 `.tether` files under
`examples/` and `tests/` still parse.

**Template ordering errors propagate.** `ordered()` already built an error
naming both offending types, but the caller discarded it with
`.unwrap_or(false)`, so a template comparing a string with `>` silently took
the untaken branch.

**Ad-block doc example compiles.** `mod adblock` is private, so
`use tetherscript::adblock::Engine` could never resolve and `cargo test --doc`
failed to compile at alpha.28. Marked `ignore`.

**`cargo clippy -- -D warnings` passes again.** alpha.28 shipped 9 warnings that
fail the gate CI runs: four unused imports, an unused `Value` import in the
template filter, and dead-code findings across the ad-block engine. The unused
imports are gone and the private module's dangling `pub use` re-exports removed.
`Engine` was kept rather than deleted — it is a real stateful layer over the
stateless `adblock_*` built-ins, just not wired to a caller — and it gained the
7 unit tests it never had.

**`cargo doc -D warnings` passes again.** The gate was failing with 33 intra-doc
link errors, also red at alpha.28: links to private items, unresolved paths for
`LineCol`/`SourceMap`/`Env`/`ProtocolError`, redundant explicit targets, and an
ambiguous `crate::rsa::verify` (both a module and a re-exported function). Fixed
in documentation only — no executable line changed.

## Known-incomplete, now documented rather than hidden

- `Rule::resource_types` is populated by the parser but never read during
  matching, so `$type` modifiers silently widen to "any resource type".
  Enforcing them needs a request-type argument threaded through
  `adblock_should_block`, which is a built-in signature change.
- `Engine` is not constructed by the built-in installation path.

## Install

```
cargo add tetherscript@0.1.0-alpha.29
```

Published to crates.io and verified installable from a clean external project.

## Validation

All ten gates in `.github/workflows/ci.yml`, **observed green on GitHub Actions**:

https://github.com/CodeTether/TetherScript/actions/runs/31129261238

| Gate | Result |
|---|---|
| `./check_file_limits.sh` | pass |
| `cargo fmt --check` | pass |
| `cargo clippy -- -D warnings` | pass |
| `cargo test --test browser_wpt_like` | pass |
| `cargo test --test browser_wpt_json` | pass |
| `cargo test --test browser_wpt_upstream` | pass |
| `cargo test` | pass (4183 passed, 0 failed locally) |
| `cargo test --doc` | pass (671 passed locally) |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | pass (was 33 errors) |
| `cargo package` | pass |

Registry evidence: `crates.io` reports `max_version` and `default_version` as
`0.1.0-alpha.29`, not yanked, crate size 2,474,531 bytes. A clean
`cargo add tetherscript@0.1.0-alpha.29` in an empty project downloaded and
compiled it.

`ci.yml` also gained `workflow_dispatch`, because pushes made with a bot token
did not always spawn an automatic run.

## Still open

- **TCP resources bypass the capability model.** `resource.tcp_listen` binds a
  socket with no grant under the default `--access-mode restricted`, the same
  class of ambient-access gap AGENTS.md documents for `fs_*` and `process_*`.
  Left deliberately: closing it is a capability-model decision.
- No UDP primitives, no bitwise operators, no `--grant-tcp` / `--grant-udp`.
