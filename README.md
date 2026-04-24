# TetherScript

A dynamically-typed scripting language with Rust-style ownership, implemented in Rust. Two interchangeable runtimes: a tree-walking interpreter (the reference implementation) and a stack-based bytecode VM.

Targeted at agent / AI workloads — small and fast to iterate on, with an ownership story that catches aliasing bugs at runtime without making you write type annotations.

TetherScript uses `.tether` source files going forward. Existing `.kl` files remain supported during the rename.

## Status: v0.0.4 — + LSP server and VSCode extension

Working: variables, functions, closures, recursion, control flow, lists, maps, strings, explicit `move`, runtime use-after-move detection, method calls, built-ins (`println`, `print`, `len`, `type_of`, `map`, `http_serve`). Both runtimes pass the same example suite with byte-identical output.

```bash
cargo build --release

# default: tree-walking interpreter
./target/release/tetherscript examples/hello.kl
./target/release/tetherscript examples/fib.kl

# bytecode VM (same semantics)
./target/release/tetherscript --vm examples/hello.kl
./target/release/tetherscript --vm examples/fib.kl

# debugging surface
./target/release/tetherscript --tokens   examples/hello.kl
./target/release/tetherscript --ast      examples/hello.kl
./target/release/tetherscript --bytecode examples/hello.kl

# language server (for editors)
./target/release/tetherscript --lsp
```

## HTTP server

TetherScript ships with a built-in blocking HTTP/1.1 server. No async runtime, no
dependencies — just `http_serve(port, handler)`:

```
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

The handler receives a request map with `method`, `path`, `query`, `headers`,
and `body`. It may return either a string (sent as `200 text/plain`) or a map
with optional `status`, `headers`, and `body`. See
[`examples/http_hello.kl`](examples/http_hello.kl) for a multi-route example.
Works in both the tree-walker and the VM.

TetherScript also hosts its own landing page:
[`examples/landing.kl`](examples/landing.kl) serves a Tailwind-styled marketing
site directly out of `http_serve`. Run `tetherscript examples/landing.kl` and open
[http://127.0.0.1:8787/](http://127.0.0.1:8787/).

### Docker

A [`Dockerfile`](Dockerfile) builds a fully-static musl binary on
`rust:alpine` and ships it on `scratch` — the final image is just the tetherscript
binary plus the `examples/` directory (≈5 MB).

```bash
docker build -t tetherscript .
docker run --rm -p 8787:8787 tetherscript
# open http://localhost:8787/

# or run a different example
docker run --rm -p 8787:8787 tetherscript /examples/http_hello.kl
```

Or via compose:

```bash
docker compose up --build
```

## Editor support

A VSCode extension lives in [`editor/vscode/`](editor/vscode/). It provides syntax highlighting and wires VSCode up to `tetherscript --lsp` for live lex / parse diagnostics. See the extension README for install instructions.

## Design

- **Dynamic types** — no annotations required, like Python.
- **Rust-style ownership, runtime-checked** — values are borrowed by default; `move x` transfers ownership; using a moved binding is a runtime panic with a clear message.
- **Copy for scalars, move for heap values** — matches Rust's rule. `let m = move n` on an int leaves `n` live; on a list it leaves a tombstone.
- **Expression-oriented blocks** — the last expression without a `;` is the block's value. `let x = if cond { a } else { b }` works.
- **Rust-like syntax** — `fn`, `let`, `let mut`, braces, `&`, `&mut`. No type annotations.
- **Errors**: `panic` for bugs, `Result<T, E>` + `?` for expected failures (parser accepts `?` and `Result`; semantics pending).

## Capabilities

TetherScript is an agent habitat. Authority to touch the world (filesystem, network,
mail) is a first-class value the program holds, passes, and attenuates.
There are no ambient I/O built-ins — the harness grants capabilities
explicitly and the program can only act through them.

```bash
tetherscript --grant-fs ./examples/cap_workspace examples/capability_fs.kl
```

Inside a program:

```
let cache = fs.narrow(ro_params)   // attenuate to read-only
fs_user.read("config.toml")?       // errors flow through Result + ?
fs.revoke()                        // kills fs and every narrowed child
```

See [`examples/capability_fs.kl`](examples/capability_fs.kl) for a walk-through
that exercises narrow, subdelegation, path-escape rejection, and cascading
revocation. Design: [docs/capabilities.md](docs/capabilities.md) (in-flight).

## What's not yet done

- `&mut` aliasing/XOR-mutability is parsed but not enforced at runtime
- `for x in iter` loops — `while` only for now
- Map / struct literal syntax (`{"key": value}`) — use `map()` + assignment
- Migrating `http_serve` / `smtp_send` onto the capability system
- Modules / imports
- Async (Tokio-hosted scheduler)
- Persistent agent identity across process restart
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
  lsp.rs       — Language Server Protocol server (JSON-RPC over stdio)
  http.rs      — blocking HTTP/1.1 server behind the `http_serve` built-in
  main.rs      — CLI entry point
examples/
  hello.kl, fib.kl, closures.kl, ownership.kl, use_after_move.kl,
  http_hello.kl, landing.kl
editor/vscode/
  VSCode extension: TextMate grammar + LSP client
```

## Roadmap

- [x] Lexer
- [x] Parser → AST
- [x] Tree-walking interpreter
- [x] Runtime ownership tracking (move/borrow, Copy scalars, use-after-move panics)
- [x] Bytecode compiler + VM (parity with tree-walker on all examples)
- [x] LSP server (lex/parse diagnostics) + VSCode extension
- [ ] `&mut` exclusivity enforcement
- [ ] `Result<T, E>` + `?` semantics
- [ ] `for x in iter` loops
- [ ] Modules + imports
- [ ] VM local-slot IR (skip env name lookup for hot loops)
- [ ] Async + scheduler (Tokio)
- [ ] Agent stdlib (HTTP, JSON, subprocess, channels)
- [ ] FFI to Rust crates
- [ ] REPL
