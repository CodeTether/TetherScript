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

The latter two were red at the `v0.1.0-alpha.28` tag; both are green now.

## Validation

All local, on this commit:

- `cargo test --release --no-fail-fast` — **4169 passed, 0 failed**
- `cargo test --doc` — 671 passed, 0 failed, 42 ignored
- `cargo fmt --check` — clean
- `cargo clippy --all-targets` — no warnings in changed files
- `./check_file_limits.sh` — passes (also clears two files that were over the
  limit at alpha.28)
- `examples/borrow_not_bitwise.tether` added, plus 12 tests across
  `src/parser/infix_tests.rs` and `tests/parser_amp_not_bitwise.rs`

No live deployment or platform upload is part of this release.

## Known gaps, unchanged

- **TCP resources bypass the capability model.** `resource.tcp_listen` binds a
  socket with no grant under the default `--access-mode restricted`. This is the
  same class of ambient-access gap AGENTS.md documents for `fs_*` and
  `process_*`. Deliberately left for a separate change, since closing it is a
  capability-model decision.
- No UDP primitives, no bitwise operators, no `--grant-tcp`/`--grant-udp`.
