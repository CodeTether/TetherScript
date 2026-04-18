# Kiln

A dynamically-typed scripting language with Rust-style ownership, implemented as a tree-walking interpreter in Rust. Bytecode VM is the next step.

Targeted at agent / AI workloads — small and fast to iterate on, with an ownership story that catches aliasing bugs at runtime without making you write type annotations.

## Status: v0.0.2 — lexer + parser + tree-walking interpreter

Working: variables, functions, closures, recursion, control flow, lists, maps, strings, explicit `move`, runtime use-after-move detection, method calls, built-ins (`println`, `print`, `len`, `type_of`, `map`).

```bash
cargo build --release
./target/release/kiln examples/hello.kl
./target/release/kiln examples/fib.kl
./target/release/kiln examples/closures.kl
./target/release/kiln examples/ownership.kl

# debugging surface
./target/release/kiln --tokens examples/hello.kl
./target/release/kiln --ast    examples/hello.kl
```

## Design

- **Dynamic types** — no annotations required, like Python.
- **Rust-style ownership, runtime-checked** — values are borrowed by default; `move x` transfers ownership; using a moved binding is a runtime panic with a clear message.
- **Copy for scalars, move for heap values** — matches Rust's rule. `let m = move n` on an int leaves `n` live; on a list it leaves a tombstone.
- **Expression-oriented blocks** — the last expression without a `;` is the block's value. `let x = if cond { a } else { b }` works.
- **Rust-like syntax** — `fn`, `let`, `let mut`, braces, `&`, `&mut`. No type annotations.
- **Errors**: `panic` for bugs, `Result<T, E>` + `?` for expected failures (parser accepts `?` and `Result`; semantics land in v0.0.3).

## What's not yet done

- `&mut` aliasing/XOR-mutability is parsed but not enforced at runtime
- `Result` / `?` operator — tokens exist, semantics pending
- `for x in iter` loops — `while` only for now
- Modules / imports
- Async (Tokio-hosted scheduler)
- Bytecode VM (the whole reason we picked this design)
- Standard library beyond the handful of built-ins
- REPL

## Layout

```
src/
  token.rs    — token types
  lexer.rs    — hand-written single-pass lexer
  ast.rs      — AST node definitions
  parser.rs   — Pratt parser for expressions, recursive descent for statements
  value.rs    — runtime values, ownership slots, environments
  interp.rs   — tree-walking interpreter
  main.rs     — CLI entry point
examples/
  hello.kl, fib.kl, closures.kl, ownership.kl, use_after_move.kl
```

## Roadmap

- [x] Lexer
- [x] Parser → AST
- [x] Tree-walking interpreter
- [x] Runtime ownership tracking (move/borrow, Copy scalars, use-after-move panics)
- [ ] `&mut` exclusivity enforcement
- [ ] `Result<T, E>` + `?` semantics
- [ ] `for x in iter` loops
- [ ] Modules + imports
- [ ] Bytecode compiler + VM
- [ ] Async + scheduler (Tokio)
- [ ] Agent stdlib (HTTP, JSON, subprocess, channels)
- [ ] FFI to Rust crates
- [ ] REPL
