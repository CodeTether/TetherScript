# TetherScript

TetherScript is a small, embeddable scripting language for Rust hosts and AI-agent
workflows. It is dynamically typed, uses Rust-like syntax, and is implemented in
Rust with two runtimes: a tree-walking interpreter and a stack-based bytecode VM.

The package, library, and binary are named `tetherscript`.

TetherScript is meant for project policy, validators, workflow glue, plugin hooks,
and other fast-changing behavior that should not require rebuilding a Rust
application.

## Status

TetherScript currently includes:

- A tree-walking interpreter used as the reference runtime.
- A stack-based bytecode VM targeting the same observable semantics.
- Dynamic values: integers, floats, booleans, strings, lists, maps, functions,
  native functions, `nil`, and `Result`.
- Variables, lexical scopes, functions, closures, recursion, `if`/`else`,
  `while`, and `for x in iterable` loops.
- Runtime ownership tracking with explicit `move` and use-after-move errors.
- `Result` values and `?` propagation for expected failures.
- Method calls on built-in values.
- JSON parsing/encoding implemented in-tree.
- Blocking HTTP client helpers and a blocking HTTP/1.1 server.
- SMTP sending support.
- Standard tools for filesystem, process, environment, path, time, Base64,
  SHA-256, and URL parsing.
- A Rust plugin host for loading TetherScript source and calling named hooks.
- A small LSP server plus VSCode extension.

The long-term direction is controlled extensibility for Rust applications:
TetherScript source is editable and inspectable, while the Rust host remains
responsible for trust, capabilities, auditing, and resource budgets.

## Quick start

```bash
cargo build --release

# Run with the reference interpreter
./target/release/tetherscript examples/hello.kl
./target/release/tetherscript examples/fib.kl

# Run with the bytecode VM
./target/release/tetherscript --vm examples/hello.kl
./target/release/tetherscript --vm examples/fib.kl

# Inspect the frontend / compiler output
./target/release/tetherscript --tokens   examples/hello.kl
./target/release/tetherscript --ast      examples/hello.kl
./target/release/tetherscript --bytecode examples/hello.kl

# Serve LSP over stdio for editors
./target/release/tetherscript --lsp
```

## Language example

```kl
fn fib(n) {
    if n < 2 {
        return n
    }
    fib(n - 1) + fib(n - 2)
}

fn main() {
    println(fib(10))
}
```

## HTTP server

TetherScript ships with a dependency-free blocking HTTP/1.1 server exposed as
`http_serve`:

```kl
fn handle(req) {
    let resp = map()
    resp.status = 200
    resp.body = "hello from tetherscript\n"
    return resp
}

fn main() {
    http_serve(8787, handle)
}
```

The handler receives a request map with `method`, `path`, `query`, `headers`, and
`body`. It may return either a string, sent as `200 text/plain`, or a map with
optional `status`, `headers`, and `body`.

## Standard tools

The runtime includes dependency-free standard tools exposed as built-ins,
including:

- `json_parse`, `json_encode`, `json_encode_pretty`
- `http_get`, `http_head`, `http_post`, `http_request`, `http_serve`
- `smtp_send`
- `fs_read`, `fs_write`, `fs_exists`, `fs_list`, `fs_mkdir`, `fs_stat`,
  `fs_remove`, `fs_rename`, `fs_copy`
- `process_run`, `process_args`, `process_pid`, `process_platform`,
  `process_arch`
- `env_get`, `cwd`, `chdir`
- `path_join`, `path_dirname`, `path_basename`, `path_extname`,
  `path_normalize`, `path_resolve`, `path_sep`
- `time_now_ms`, `sleep_ms`
- `sha256_hex`, `base64_encode`, `base64_decode`, `url_parse`

See [`docs/standard-tools.md`](docs/standard-tools.md) for more detail.

## Plugin embedding

TetherScript can be embedded as a local plugin language. A Rust host can load
TetherScript source, grant host-provided authority, and call named functions as
hooks.

This is the intended boundary:

```text
Rust host / agent system -> TetherScript hook -> granted host capability -> structured result
```

See [`docs/tetherscript-and-codetether.md`](docs/tetherscript-and-codetether.md)
for the CodeTether integration model.

## Editor support

A VSCode extension lives in [`editor/vscode/`](editor/vscode/). It provides syntax
highlighting and connects VSCode to `tetherscript --lsp` for live lex/parse
diagnostics.

## Design principles

- **Dynamic types** — no type annotations are required.
- **Rust-like syntax** — `fn`, `let`, `let mut`, braces, `&`, `&mut`, `move`,
  and expression-oriented blocks.
- **Runtime-checked ownership** — heap values can be moved; use-after-move is
  reported at runtime.
- **Copy scalars, move heap values** — scalar values remain usable after `move`;
  lists/maps/strings transfer ownership.
- **Recoverable errors** — `Result` and `?` handle expected failure; `panic` is
  for bugs.
- **Embeddable host boundary** — host effects should flow through explicit host
  integration rather than unreviewable shell glue.
- **Zero-dependency goal** — the runtime avoids third-party crates unless a
  future governance decision justifies them.

## What is not done yet

- Runtime `&mut` aliasing / XOR-mutability enforcement.
- Modules and imports.
- Formatter and REPL.
- More complete LSP features such as completions, hover, go-to-definition, and
  exact spans.
- Capability audit logs and richer resource budgets.
- Moving ambient host tools behind explicit capabilities where practical.
- VM local-slot optimization; variable lookup is still name-based.
- Async scheduler.

## Repository layout

```text
src/
  ast.rs         — AST node definitions
  bytecode.rs    — bytecode instruction/chunk/function types
  capability.rs  — capability trait/object model
  compiler.rs    — AST to bytecode compiler
  http.rs        — HTTP client/server built-ins
  interp.rs      — tree-walking interpreter
  json.rs        — in-tree JSON parser/encoder
  lexer.rs       — lexer
  lib.rs         — library surface for embedding
  lsp.rs         — LSP server
  output.rs      — output capture helpers
  parser.rs      — parser
  plugin.rs      — Rust host plugin API
  smtp.rs        — SMTP support
  system.rs      — filesystem/process/env/path/time/hash/base64/url tools
  token.rs       — token types
  value.rs       — runtime values, ownership slots, environments
  vm.rs          — bytecode VM
examples/
  hello.kl, fib.kl, closures.kl, ownership.kl, use_after_move.kl
editor/vscode/
  VSCode grammar and LSP client
docs/
  standard tools and CodeTether integration notes
```

## Roadmap

- [x] Lexer
- [x] Parser and AST
- [x] Tree-walking interpreter
- [x] Runtime ownership tracking
- [x] Bytecode compiler and VM
- [x] LSP server and VSCode extension
- [x] `Result` and `?` semantics
- [x] `for x in iterable` loops
- [x] JSON support
- [x] HTTP client/server support
- [x] Standard filesystem/process/env/path/time/hash/base64/url tools
- [x] Rust embedding/plugin host
- [ ] Runtime `&mut` exclusivity enforcement
- [ ] Modules and imports
- [ ] Plugin and capability manifests as stable formats
- [ ] Audit stream for capability calls
- [ ] Formatter
- [ ] REPL
- [ ] Async scheduler
