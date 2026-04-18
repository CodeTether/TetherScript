# rk

A dynamically-typed scripting language with Rust-style ownership, implemented as a bytecode VM in Rust. Targeted at agent/AI workloads.

## Status: v0.0.1 — lexer only

This is the skeleton. Right now `rk` can tokenize source and that's it. Parser, compiler, and VM come next.

```bash
cargo run -- examples/hello.rk
```

## Design

- **Dynamic types**, like Python. No annotations required.
- **Rust-style ownership**, runtime-checked. Values are borrowed by default; `move x` transfers ownership; using a moved value panics.
- **Expression-oriented**. Blocks are expressions; the last expr (without a `;`) is the value.
- **Rust-like syntax**: `fn`, `let`, `let mut`, braces, `&`, `&mut`.
- **Errors**: `panic` for bugs, `Result<T, E>` + `?` for expected failures.

## Roadmap

- [x] Lexer
- [ ] Parser → AST
- [ ] Tree-walking interpreter (dev loop, testing)
- [ ] Bytecode compiler + VM
- [ ] Runtime ownership tracking (move/borrow states on values)
- [ ] Standard library (print, strings, lists, maps, file I/O)
- [ ] Async + scheduler (Tokio-hosted)
- [ ] Agent-oriented stdlib (HTTP, JSON, subprocess, channels)
- [ ] FFI to Rust crates
