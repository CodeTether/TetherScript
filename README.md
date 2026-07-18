# tetherscript

[![crates.io](https://img.shields.io/crates/v/tetherscript.svg)](https://crates.io/crates/tetherscript)
[![crates.io downloads](https://img.shields.io/crates/d/tetherscript.svg)](https://crates.io/crates/tetherscript)
[![docs.rs](https://img.shields.io/docsrs/tetherscript.svg)](https://docs.rs/tetherscript)

tetherscript is a dynamically typed, ownership-aware language for portable tools,
AI-agent workflows, and embeddable application logic. It uses Rust-like syntax and
is currently implemented in Rust with a tree-walking reference interpreter and a
stack-based bytecode VM.

The package, library, and binary are named `tetherscript`.

The long-term goal is to compete in the space served by Zig: standalone native
tools, predictable deployment, cross-compilation, and direct native interoperability.
tetherscript is not intended to replace Rust. Rust remains the bootstrap and host
implementation language while tetherscript develops its own compiler stack.

## CLI scripts and standalone launchers

Run a script with command-line arguments:

```bash
tetherscript run examples/cli_args.tether -- --name Riley
```

Inside the script, `env_args()` returns the process arguments passed after the
script path or after `--`:

```tether
fn main() {
    let args = env_args()
    println("argc", args.len())
    for arg in args { println("arg", arg) }
}
```

Build a standalone launcher executable that embeds the `.tether` source and runs
it with the bytecode VM:

```bash
tetherscript build examples/cli_args.tether -o mycli
./mycli --name Riley
```

The built launcher forwards its own process arguments to `env_args()`. It is a
self-contained runner for the script; internally it compiles the embedded source
to tetherscript bytecode on startup before VM execution.

## Status

tetherscript `0.1.0-alpha.18` is the current release candidate for crates.io.

tetherscript currently includes:

- A tree-walking interpreter used as the reference runtime.
- A stack-based bytecode VM targeting the same observable semantics, with local
  slots for function parameters and locals plus constant-pool deduplication.
- An experimental, tetherscript-owned SSA-like IR for straight-line functions,
  with structural verification and `tetherscript inspect --ir <file>` output.
  Native machine-code generation is not part of this initial slice.
- Script CLI arguments via `env_args()` for `tetherscript run <file> -- [args...]`.
- Standalone executable launchers via `tetherscript build <file.tether> -o <output>`.
- Dynamic values: integers, floats, booleans, strings, lists, maps, functions,
  native functions, `nil`, and `Result`.
- Variables, lexical scopes, functions, closures, recursion, `if`/`else`,
  `while`, and `for x in iterable` loops.
- Runtime ownership tracking with explicit `move` and use-after-move errors.
- `Result` values and `?` propagation for expected failures.
- Method calls on built-in values.
- JSON parsing/encoding implemented in-tree.
- Blocking HTTP client helpers, a blocking HTTP/1.1 handler server, and a
  native cached static-file server.
- Experimental browser primitives for parsing small HTML fragments, applying
  simple CSS rules, computing block layout, and rendering deterministic text
  display lists.
- A dependency-free JavaScript host for those browser primitives, with `document`,
  `window`, selectors, attributes, inline `<script>` execution, text/attribute
  mutation, basic DOM tree mutation APIs, synchronous DOM events,
  `location`/`navigator` globals, `this`, `typeof`, function expressions,
  classic `for` loops for NodeList-style iteration, and deterministic
  timers, microtasks, Promise reactions, `await`, route-backed module imports,
  `fetch`, and `XMLHttpRequest` lifecycle draining after scripts.
- Browser JavaScript compatibility for common production bundle constructs used
  by Vite/React login flows, including arrow functions, classes, optional
  chaining, nullish coalescing, dynamic import, spread/rest, template literals,
  regex route matching, Promise adoption, and XHR response fields.
- Native browser-agent APIs for agent validation workflows, including page
  loading from provided HTML, external resource registration, resource
  validation, stylesheet/script inlining, classic and module script execution,
  route-backed static module import loading, default ESM import/export
  rewriting, dynamic `import()` resolution/rejection, DOM assertions,
  screenshots/traces, production debug reports, HAR-style network exports,
  source-mapped runtime error locations and
  generated async/module stack frames, unhandled promise rejection reporting,
  React root/render/hydration diagnostics, classified runtime exceptions with
  separate CORS and route-abort categories, computed-style plus layout evidence
  for rendered elements,
  React-style controlled form interaction coverage, fetch/XHR server-cookie
  propagation, redirect following, CORS preflight validation, credential modes,
  routed page-resource loading for auth flows, routed top-level navigation for
  `location`, anchor, and form commits, and explicit tests that reject external
  browser engines and remote-control drivers as required browser backends.
- DOM/event parity coverage now includes composed shadow-root event paths that
  bubble through the host chain with inspectable `composedPath()` output.
- Element child collections now behave live for `children` and `childNodes`
  after DOM mutation.
- Document-wide collections now behave live for `getElementsBy*` calls and
  named HTMLCollections such as `document.forms`.
- DOM click defaults now include label activation, native anchor `location`
  updates, browser-shaped `form.submit()` versus `requestSubmit()` behavior,
  and submitter-specific `requestSubmit(submitter)` form data.
- WPT-like browser fixtures now cover DOM events, Selectors API, Fetch/CORS,
  module scripts, CSS/layout, timers/microtasks, Web Storage, HTML tree
  construction, form defaults, navigation history, context storage/cookies,
  keyboard/pointer interaction, focus order, and file upload/download behavior,
  realtime channels, permissions/media APIs, and dialog/clipboard behavior,
  frames/window messaging, security policy, canvas/WebGL, and accessibility
  snapshots, service workers/cache storage, IndexedDB, selection,
  screenshots/visual diff, and page trace/persistence behavior, including
  negative/error cases, through `cargo test --test browser_wpt_like`.
- SMTP sending support.
- Standard tools for filesystem, process, environment, path, time, Base64,
  SHA-256, and URL parsing.
- A self-hosting agent TUI example with hot reload, persistent conversation
  state, and a source-check gate for agentic self-improvement.
- A Rust plugin host for loading tetherscript source and calling named hooks.
- A small LSP server plus VSCode extension.

Embedding remains a first-class use case: tetherscript source is editable and
inspectable, while a Rust host can remain responsible for trust, capabilities,
auditing, and resource budgets.

## Compiler architecture

The interpreter defines observable language semantics. The native compiler track
keeps those semantics in a tetherscript-owned pipeline rather than making LLVM the
language's architectural boundary:

```text
.tether source -> AST -> Tether IR -> machine IR -> native object -> executable
```

Today, Tether IR lowers and verifies straight-line functions with constants,
bindings, dynamic arithmetic and comparisons, explicit moves, named calls, and
returns. Inspect it with:

```bash
tetherscript inspect --ir examples/ir_arithmetic.tether
```

Control-flow lowering, optimization passes, machine IR, register allocation,
object emission, and cross-compilation are roadmap work. The existing `build`
command creates a standalone bytecode launcher; it does not yet emit native code.

## Quick start

```bash
cargo install tetherscript --version 0.1.0-alpha.18

cargo build --release

# Run with the bytecode VM (default)
./target/release/tetherscript run examples/hello.tether
./target/release/tetherscript run examples/fib.tether

# Run with the reference interpreter
./target/release/tetherscript run --interp examples/hello.tether
./target/release/tetherscript run --interp examples/fib.tether

# Inspect the frontend / compiler output
./target/release/tetherscript inspect --tokens   examples/hello.tether
./target/release/tetherscript inspect --ast      examples/hello.tether
./target/release/tetherscript inspect --ir       examples/ir_arithmetic.tether
./target/release/tetherscript inspect --bytecode examples/hello.tether
./target/release/tetherscript inspect --bytecode-visual examples/fib.tether

# Serve LSP over stdio for editors
./target/release/tetherscript lsp
```

## Bytecode visualizer

The default `run` command compiles source into bytecode and executes it on the
stack VM. The visualizer makes that pipeline inspectable:

```bash
./target/release/tetherscript inspect --bytecode-visual examples/fib.tether
```

Example output:

```text
bytecode visualizer
chunk main
  names (2)
    n000 = fib
    n001 = main
  code (6)
    0000  MakeFn(0)
    0001  DefLet(0, false)
    0002  MakeFn(1)
    0003  DefLet(1, false)
    0004  Nil
    0005  Return
  chunk p000 fn fib(n)
    code (22)
      0000  GetName(0)
      0001  Const(0)
      0002  Lt
      0003  JumpIfFalse(4) -> 0008
      ...
```

Use `--bytecode` for the raw debug dump and `--bytecode-visual` for teaching,
reviews, and understanding how source maps to VM instructions.

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

tetherscript ships with a dependency-free blocking HTTP/1.1 server exposed as
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

For static sites, `http_serve_static(port, root_dir)` preloads files through the
granted `fs` capability and serves precomputed responses from native Rust without
calling a tetherscript handler per request:

```kl
fn main() {
    http_serve_static(8788, "dist")
}
```

Run it with a filesystem grant scoped to the site root:

```bash
./target/release/tetherscript run --grant-fs examples/content_site \
  examples/static_site_server_native.tether
```

## Standard tools

The runtime includes dependency-free standard tools exposed as built-ins,
including:

- `json_parse`, `json_encode`, `json_encode_pretty`
- `http_get`, `http_head`, `http_post`, `http_request`, `http_serve`,
  `http_serve_static`
- `browser_parse_html`, `browser_parse_css`, `browser_styles`,
  `browser_query_selector`, `browser_text_content`, `browser_snapshot`,
  `browser_layout`, `browser_display_list`, `browser_render`
- `js_eval`, `browser_eval_js`, `browser_run_scripts`,
  `browser_compatibility_report`
- `smtp_send`
- `fs_read`, `fs_write`, `fs_exists`, `fs_list`, `fs_mkdir`, `fs_stat`,
  `fs_remove`, `fs_rename`, `fs_copy`
- `process_run`, `process_args`, `process_pid`, `process_platform`,
  `process_arch`, `process_list`, `process_kill`
- `tui_size`, `tui_render`, `tui_present`, `tui_read_event`,
  `tui_clear`, `tui_cursor`, `tui_alt_screen`, `tui_move_to`
- `stdio_read`, `stdio_write`, `stdio_write_err`, `jsonrpc_request`,
  `jsonrpc_response`, `jsonrpc_error`, `jsonrpc_notify`
- `env_get`, `cwd`, `chdir`
- `path_join`, `path_dirname`, `path_basename`, `path_extname`,
  `path_normalize`, `path_resolve`, `path_sep`
- `time_now_ms`, `sleep_ms`
- `sha256_hex`, `base64_encode`, `base64_decode`, `url_parse`

See [`docs/standard-tools.md`](docs/standard-tools.md) for more detail. See
[`docs/agent-tui.md`](docs/agent-tui.md) for the user-land agent TUI pattern.
Provider grants can also be bootstrapped from CodeTether-style Vault KV v2
secrets with `--grant-provider-vault <provider-id>`.
For local agent-style runs, `--access-mode full` grants the current directory
and auto-loads a default provider from Vault first, then local environment
fallback unless `CODETETHER_DISABLE_ENV_FALLBACK=1` is set.

## Agentic self-improvement

`examples/agent_tui.tether` is the reference self-hosting agent loop. It is a
real `.tether` script, not a built-in Rust agent: the model can inspect the
workspace, call file tools, edit the TUI source, and let the runner hot-reload
the changed script in the same terminal process.

The improvement guarantee is deliberately narrow and testable. tetherscript does
not claim that an agent can prove every self-edit is globally better. Instead,
the TUI guarantees that a self-edit is not accepted for reload unless the edited
source passes the project's source check:

```text
agent edits examples/agent_tui.tether
-> TUI detects the source changed at a turn boundary
-> TUI runs: tetherscript check <script>
-> passing candidate reloads
-> failing candidate restores the previous source and keeps running
```

This makes self-improvement an acceptance-gated workflow rather than an
unbounded rewrite loop. The regression coverage in `tests/agent_tui.rs` includes
both the successful self-reload path and an invalid self-edit that must be
rejected, restored, and prevented from writing a reload marker.

## Plugin embedding

tetherscript can be embedded as a local plugin language. A Rust host can load
tetherscript source, grant host-provided authority, and call named functions as
hooks.

This is the intended boundary:

```text
Rust host / agent system -> tetherscript hook -> granted host capability -> structured result
```

See [`docs/tetherscript-and-codetether.md`](docs/tetherscript-and-codetether.md)
for the CodeTether integration model.

## Editor support

A VSCode extension lives in [`editor/vscode/`](editor/vscode/). It provides syntax
highlighting and connects VSCode to `tetherscript lsp` for live lex/parse
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
- Complete Test262 and Web Platform Tests coverage. The JavaScript/browser
  runtime is an in-tree, zero-dependency compatibility track and currently
  implements a practical DOM/JS subset rather than every web standard.
- Full browser networking, navigation, DOM, CSS, rendering, JS, and Web API
  parity. The browser track is intended to become a native full-parity browser
  for agents, with conformance tracked explicitly rather than delegated to an
  external browser engine.
- Capability audit logs and richer resource budgets.
- Moving ambient host tools behind explicit capabilities where practical.
- Async scheduler.
- Tether IR lowering for control flow, closures, mutable slots, and all ownership
  operations.
- Optimization passes, machine IR, instruction selection, register allocation,
  native object emission, and debug information.
- Cross-compilation and stable C ABI interoperability.

## Repository layout

```text
src/
  ast.rs         — AST node definitions
  browser.rs     — experimental HTML/CSS parser, layout, and text renderer
  browser_js.rs  — dependency-free browser JavaScript DOM host bindings
  bytecode.rs    — bytecode instruction/chunk/function types
  capability.rs  — capability trait/object model
  compiler.rs    — AST to bytecode compiler
  http.rs        — HTTP built-in module surface
  http_client.rs — plain HTTP client helpers
  http_server.rs — dynamic handler HTTP server
  http_static/   — native cached static-file HTTP server
  interp.rs      — tree-walking interpreter
  ir/            — Tether IR model, lowering, verifier, and renderer
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
  hello.tether, fib.tether, closures.tether, ownership.tether, use_after_move.tether
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
- [x] Initial straight-line Tether IR, verifier, and textual renderer
- [x] LSP server and VSCode extension
- [x] `Result` and `?` semantics
- [x] `for x in iterable` loops
- [x] JSON support
- [x] HTTP client/server support
- [x] Standard filesystem/process/env/path/time/hash/base64/url tools
- [x] Experimental browser parser/layout/text renderer
- [x] Experimental dependency-free browser JavaScript host bindings
- [x] Native production debug report for bundled UI validation
- [x] Rust embedding/plugin host
- [ ] Native full browser parity
- [ ] Full AST-to-Tether-IR lowering
- [ ] Optimization pass framework and machine IR
- [ ] First native backend and object-file emission
- [ ] Cross-compilation and stable C ABI interoperability
- [ ] Runtime `&mut` exclusivity enforcement
- [ ] Modules and imports
- [ ] Plugin and capability manifests as stable formats
- [ ] Audit stream for capability calls
- [ ] Formatter
- [ ] REPL
- [ ] Async scheduler
