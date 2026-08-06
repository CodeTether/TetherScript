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

## Validation

All ten gates in `.github/workflows/ci.yml`, run locally on the tagged commit.
CI did not trigger a run for these pushes, so these are local results, not
CI-observed:

| Gate | Result |
|---|---|
| `./check_file_limits.sh` | pass |
| `cargo fmt --check` | pass |
| `cargo clippy -- -D warnings` | pass |
| `cargo test --test browser_wpt_like` | 40 passed |
| `cargo test --test browser_wpt_json` | 1 passed |
| `cargo test --test browser_wpt_upstream` | 3 passed |
| `cargo test` | **4183 passed, 0 failed** |
| `cargo test --doc` | 671 passed, 0 failed |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | pass (was 33 errors) |
| `cargo package` | packaged and verified |

No live deployment or platform upload is part of this release.

## Still open

- **TCP resources bypass the capability model.** `resource.tcp_listen` binds a
  socket with no grant under the default `--access-mode restricted`, the same
  class of ambient-access gap AGENTS.md documents for `fs_*` and `process_*`.
  Left deliberately: closing it is a capability-model decision.
- No UDP primitives, no bitwise operators, no `--grant-tcp` / `--grant-udp`.
