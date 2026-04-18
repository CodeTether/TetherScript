# Kiln

A dynamically-typed scripting language with Rust-style ownership, implemented in Rust. Two interchangeable runtimes: a tree-walking interpreter (the reference implementation) and a stack-based bytecode VM.

Targeted at agent / AI workloads — small and fast to iterate on, with an ownership story that catches aliasing bugs at runtime without making you write type annotations.

## Status: v0.0.3 — lexer + parser + tree-walker + bytecode VM

Working: variables, functions, closures, recursion, control flow, lists, maps, strings, explicit `move`, runtime use-after-move detection, method calls, built-ins (`println`, `print`, `len`, `type_of`, `map`). Both runtimes pass the same example suite with byte-identical output.

```bash
cargo build --release

# default: tree-walking interpreter
./target/release/kiln examples/hello.kl
./target/release/kiln examples/fib.kl

# bytecode VM (same semantics)
./target/release/kiln --vm examples/hello.kl
./target/release/kiln --vm examples/fib.kl

# debugging surface
./target/release/kiln --tokens   examples/hello.kl
./target/release/kiln --ast      examples/hello.kl
./target/release/kiln --bytecode examples/hello.kl
```

## Design

- **Dynamic types** — no annotations required, like Python.
- **Rust-style ownership, runtime-checked** — values are borrowed by default; `move x` transfers ownership; using a moved binding is a runtime panic with a clear message.
- **Copy for scalars, move for heap values** — matches Rust's rule. `let m = move n` on an int leaves `n` live; on a list it leaves a tombstone.
- **Expression-oriented blocks** — the last expression without a `;` is the block's value. `let x = if cond { a } else { b }` works.
- **Rust-like syntax** — `fn`, `let`, `let mut`, braces, `&`, `&mut`. No type annotations.
- **Errors**: `panic` for bugs, `Result<T, E>` + `?` for expected failures (parser accepts `?` and `Result`; semantics pending).

## What's not yet done

- `&mut` aliasing/XOR-mutability is parsed but not enforced at runtime
- `Result` / `?` operator — tokens exist, semantics pending
- `for x in iter` loops — `while` only for now
- Modules / imports
- Async (Tokio-hosted scheduler)
- Standard library beyond the handful of built-ins
- REPL
- Local-slot optimization in the VM (variable lookup is still by name)

## Layout

```
src/
  token.rs     — token types
  lexer.rs     — hand-written single-pass lexer
  ast.rs       — AST node definitions
  parser.rs    — Pratt parser for expressions, recursive descent for statements
  value.rs     — runtime values, ownership slots, environments
  interp.rs    — tree-walking interpreter (reference impl)
  bytecode.rs  — Instr / Chunk / FnProto / VmFnObj
  compiler.rs  — AST -> bytecode
  vm.rs        — stack-based bytecode VM
  main.rs      — CLI entry point
examples/
  hello.kl, fib.kl, closures.kl, ownership.kl, use_after_move.kl
```

## Roadmap

- [x] Lexer
- [x] Parser → AST
- [x] Tree-walking interpreter
- [x] Runtime ownership tracking (move/borrow, Copy scalars, use-after-move panics)
- [x] Bytecode compiler + VM (parity with tree-walker on all examples)
- [ ] `&mut` exclusivity enforcement
- [ ] `Result<T, E>` + `?` semantics
- [ ] `for x in iter` loops
- [ ] Modules + imports
- [ ] VM local-slot IR (skip env name lookup for hot loops)
- [ ] Async + scheduler (Tokio)
- [ ] Agent stdlib (HTTP, JSON, subprocess, channels)
- [ ] FFI to Rust crates
- [ ] REPL
