# Kiln

Kiln is a small, embeddable scripting language for Rust hosts and AI-agent workflows.
It is dynamically typed, uses Rust-like syntax, and keeps authority explicit through
host-granted capabilities.

Kiln is meant for project policy, validators, workflow glue, plugin hooks, and other
fast-changing behavior that should not require rebuilding a Rust application.

## Status

Kiln currently includes:

- A tree-walking interpreter used as the reference runtime.
- A stack-based bytecode VM targeting the same observable semantics.
- Dynamic values: integers, floats, booleans, strings, lists, maps, functions, native functions, `nil`, and `Result`.
- Variables, lexical scopes, functions, closures, recursion, `if`/`else`, `while`, and `for x in iterable` loops.
- Runtime ownership tracking with explicit `move` and use-after-move errors.
- `Result` values and `?` propagation for expected failures.
- Method calls on built-in values.
- JSON parsing/encoding implemented in-tree.
- Blocking HTTP client and HTTP/1.1 server support.
- SMTP sending support.
- Standard tools for filesystem, process, environment, path, time, Base64, SHA-256, and URL parsing.
- Host-granted filesystem and HTTP capabilities with narrowing and revocation.
- A Rust plugin host for loading Kiln source and calling named hooks.
- A small LSP server plus VSCode extension.

The long-term direction is controlled extensibility for Rust applications: Kiln source
is editable and inspectable, while the Rust host remains responsible for trust,
capabilities, auditing, and resource budgets.

## Quick start

```bash
cargo build --release

# Run with the reference interpreter
./target/release/kiln examples/hello.kl
./target/release/kiln examples/fib.kl

# Run with the bytecode VM
./target/release/kiln --vm examples/hello.kl
./target/release/kiln --vm examples/fib.kl

# Inspect the frontend / compiler output
./target/release/kiln --tokens   examples/hello.kl
./target/release/kiln --ast      examples/hello.kl
./target/release/kiln --bytecode examples/hello.kl

# Serve LSP over stdio for editors
./target/release/kiln --lsp
```

If your local checkout still uses the transitional binary name, replace `kiln` with
`tetherscript` in the commands above.

## Capabilities

Kiln treats authority as a value. A host grants a capability, Kiln code passes or
narrows it, and revocation fails closed. This is the core safety model for agent- and
plugin-written source.

```bash
./target/release/kiln --grant-fs ./examples/cap_workspace examples/capability_fs.kl
./target/release/kiln --grant-fs . examples/policy.tether
./target/release/kiln --vm --grant-fs . examples/policy.tether
```

Inside a Kiln program:

```kl
let cache = fs.narrow(ro_params)   // attenuate to read-only
fs_user.read("config.toml")?       // errors flow through Result + ?
fs.revoke()                        // kills fs and narrowed children
```

See:

- [`examples/capability_fs.kl`](examples/capability_fs.kl) for filesystem grants, narrowing, subdelegation, path-escape rejection, and cascading revocation.
- [`examples/capability_http.kl`](examples/capability_http.kl) for host-granted HTTP authority.
- [`examples/policy.tether`](examples/policy.tether) for a compact policy script.
- [`examples/embed_policy.rs`](examples/embed_policy.rs) for calling Kiln from Rust with an explicit grant.

## Plugin hooks

Kiln can be embedded as a local plugin language. A Rust host can load Kiln source,
grant narrow authority, and call named functions as hooks.

CLI helpers exist for exercising the plugin surface:

```bash
./target/release/kiln --plugin examples/kiln_extension.kl metadata
./target/release/kiln --plugin examples/kiln_extension.kl validate '{"path":"README.md"}'
./target/release/kiln --codetether-manifest examples/kiln_extension.kl
```

This is the intended boundary:

```text
Rust host / agent system -> Kiln hook -> granted capability -> structured result
```

## HTTP server

Kiln ships with a dependency-free blocking HTTP/1.1 server exposed as `http_serve`:

```kl
fn handle(req) {
    let resp = map()
    resp.status = 200
    resp.body = "hello from kiln\n"
    return resp
}

fn main() {
    http_serve(8787, handle)
}
```

The handler receives a request map with `method`, `path`, `query`, `headers`, and
`body`. It may return either a string, sent as `200 text/plain`, or a map with
optional `status`, `headers`, and `body`.

See [`examples/http_hello.kl`](examples/http_hello.kl).

## Docker

A [`Dockerfile`](Dockerfile) builds a static musl binary and ships it in a small final
image with the examples directory.

```bash
docker build -t kiln .
docker run --rm -p 8787:8787 kiln

# Or run a specific example
docker run --rm -p 8787:8787 kiln /examples/http_hello.kl
```

Or with Compose:

```bash
docker compose up --build
```

## Editor support

A VSCode extension lives in [`editor/vscode/`](editor/vscode/). It provides syntax
highlighting and connects VSCode to the Kiln LSP server for live lex/parse diagnostics.

## Design principles

- **Dynamic types** — no type annotations are required.
- **Rust-like syntax** — `fn`, `let`, `let mut`, braces, `&`, `&mut`, `move`, and expression-oriented blocks.
- **Runtime-checked ownership** — heap values can be moved; use-after-move is reported at runtime.
- **Copy scalars, move heap values** — scalar values remain usable after `move`; lists/maps/strings transfer ownership.
- **Recoverable errors** — `Result` and `?` handle expected failure; `panic` is for bugs.
- **Explicit authority** — host effects should flow through granted capabilities rather than ambient access.
- **Zero-dependency goal** — the runtime avoids third-party crates unless a future governance decision justifies them.

## What is not done yet

- Runtime `&mut` aliasing / XOR-mutability enforcement.
- Modules and imports.
- Formatter and REPL.
- More complete LSP features such as completions, hover, go-to-definition, and exact spans.
- Capability audit logs and richer resource budgets.
- Moving all ambient host tools behind explicit capabilities where practical.
- VM local-slot optimization; variable lookup is still name-based.
- Async scheduler.

## Repository layout

```text
src/
  ast.rs         — AST node definitions
  bytecode.rs    — bytecode instruction/chunk/function types
  capability.rs  — capability trait/object model
  cli.rs         — CLI entry point logic
  codetether.rs  — manifest adapter for CodeTether-style plugin discovery
  compiler.rs    — AST to bytecode compiler
  experiment.rs  — source-emission experiment harness
  fs_cap.rs      — filesystem capability implementation
  http.rs        — HTTP client/server built-ins
  http_cap.rs    — HTTP capability implementation
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
  capability_fs.kl, capability_http.kl, http_hello.kl, json.kl
  policy.tether, kiln_extension.kl, smtp_local.kl, send_riley.kl
  embed_policy.rs
editor/vscode/
  VSCode grammar and LSP client
docs/
  design and ecosystem notes
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
- [x] Filesystem and HTTP capabilities
- [x] Rust embedding/plugin host
- [ ] Runtime `&mut` exclusivity enforcement
- [ ] Modules and imports
- [ ] Plugin and capability manifests as stable formats
- [ ] Audit stream for capability calls
- [ ] Formatter
- [ ] REPL
- [ ] Async scheduler
