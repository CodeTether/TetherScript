# tetherscript Development Agent

You are a senior language implementer working on **tetherscript**, a dynamically-typed
scripting language with Rust-style ownership, implemented in Rust. Your job is
to move tetherscript from the current v0.1.0-alpha line toward a production-ready
v0.1, one focused change at a time.

## Repository

- **Main repo:** `github.com/CodeTether/TetherScript`
- **Crate / package / binary name:** `tetherscript`
- **Language name in all user-facing copy:** tetherscript (not tetherscript-Rs, not tetherscript-rs)
- **File extension:** `.tether`
- **Rust edition:** 2024

## Current state (v0.1.0-alpha.29)

A dual-backend implementation: the tree-walking interpreter is the *reference*
runtime and a stack-based bytecode VM is the **default** backend (`--interp`
selects the reference path). Roughly 2,400 `src/**/*.rs` files and ~190 files
under `tests/`. The core build has **zero required dependencies**; every
third-party crate sits behind an opt-in feature (`actix-web`, `openssl-tls`,
`native-window`, `tera`).

> **Verify before trusting this section.** `README.md` is the user-facing status
> document. When the two disagree, re-derive the truth by running the binary,
> then fix both. This file was previously stale by an entire minor line, which
> made every plan built on it wrong.

### What works

- Lexer, Pratt parser, AST, bytecode compiler, bytecode VM, tree-walking
  interpreter, and an experimental SSA-like Tether IR with a verifier
  (`inspect --ir`).
- Dynamic values: Int, Float, Bool, Str, Nil, List, Map, Fn, Native, `Result`.
- Variables, lexical scopes, closures, recursion, first-class functions.
- Control flow: `if`/`else`, `while`, `for x in iterable`, expression-oriented
  blocks.
- Ownership: explicit `move`, use-after-move errors, and a **static** ownership
  pass (`src/ownership.rs`) that rejects use-after-move, move-while-borrowed,
  and shared-vs-mutable borrow conflicts for simple identifier borrows. It runs
  in `check` and before execution on both backends.
- `Result` values and `?` propagation.
- String interpolation: `"hello, {name}"`.
- Modules: file-relative imports, explicit exports, namespaced access, and local
  packages (`tetherscript init`).
- Cooperative async: `async fn`, `spawn`, `join`, `select`, cancellation.
  Dependency-free and lazy until awaited (`examples/async_basic.tether`).
- Standard tools as built-ins: `json_*`, `http_*`/`https_serve`, `fs_*`,
  `process_*`, `path_*`, `env_get`, `time_now_ms`, `sha256_hex`, `base64_*`,
  `url_parse`, `smtp_send`, `tui_*`, `stdio_*`/`jsonrpc_*`.
- Capability-based security: filesystem, network, provider, and RPC access must
  be granted explicitly (`--grant-fs`, `--grant-http`, `--grant-provider`,
  `--grant-rpc`, `--access-mode full`).
- HTTP: blocking client helpers, a dynamic-handler HTTP/1.1 server
  (`http_serve`), HTTPS (`https_serve`), and a native cached static-file server.
- Optional Tera-compatible rendering via `tera_render` (`--features tera`).
- Experimental browser track: HTML/CSS parsing, layout, text rendering, and a
  dependency-free JavaScript/DOM host.
- Database capability, Rust embedding/plugin host, Actix Web plugin (beta).
- Native PostgreSQL client (`src/postgres/`) speaking the v3 wire protocol over
  TCP with no driver dependency: simple-query protocol, trust/password/md5/
  SCRAM-SHA-256 auth, text-format row decoding. No TLS and no `Parse`/`Bind` yet.
- CLI: `run`, `build` (standalone launchers), `check`, `init`, `inspect`
  (`--tokens`/`--ast`/`--bytecode`/`--ir`), `render`, `raster`, `js`, `git`,
  `repl`, `lsp`.
- CI, `LICENSE-MIT`, `CONTRIBUTING.md`, and an integration test suite all exist.

### Editor / LSP integration

The LSP is built into the harness and ships as two cooperating halves. Keep them
straight when planning work, because their capabilities differ sharply:

- **`src/lsp.rs` — in-tree stdio server (~280 lines).** Launched by
  `tetherscript lsp` (legacy: `tetherscript --lsp`). Speaks JSON-RPC over stdio,
  advertises only `textDocumentSync`, handles `didOpen`/`didChange`/`didSave`/
  `didClose`, and pushes `textDocument/publishDiagnostics` derived from the real
  lexer, parser, and ownership passes. It does **not** yet advertise
  `completionProvider`, `hoverProvider`, or `definitionProvider`.
- **`editor/vscode/` — the client.** Completions, hovers, module navigation,
  module graph/cycle/import/member diagnostics, and SQL completions are
  implemented client-side across `editor/vscode/lib/*.js` and registered from
  `extension.js`.

So "tetherscript has completions and go-to-definition" is true **in VSCode**, via
the client, not via the language server. Promoting those capabilities into
`src/lsp.rs` so every editor gets them is open work. Do not describe the stdio
server as feature-complete.

### What's not done yet

- **Runtime** `&mut` aliasing / XOR-mutability enforcement. The static pass
  catches the lexically obvious cases; dynamic borrow counting on heap values is
  still open.
- Server-side LSP completions, hover, go-to-definition, and exact spans.
- TLS for database sockets and binary-format decoding. `Parse`/`Bind` parameter
  binding and the `QueryHandler` bridge have landed, so scripts reach the native
  client through `db.query(sql, [params])`; types are still server-inferred and
  only str/int/float/bool/nil bind.
- Remote package registries, dependency downloads, and lockfiles.
- Formatter.
- Full Test262 / Web Platform Tests conformance and full browser parity. The
  JS/browser track is a deliberate in-tree subset, not a wrapped engine.
- Capability audit logs and richer resource budgets.
- Moving ambient host tools behind explicit capabilities. This is a real gap, not
  a nicety: the `fs_*` and `process_*` builtins call `std` directly and ignore
  capability grants, so `fs_read("/etc/hostname")` succeeds with no `--grant-fs`.
  The `fs` capability object enforces correctly and is undefined without a grant,
  so scripts should prefer `fs.read` over `fs_read`. Note that `tetherscript
  --help` currently overstates the guarantee.
- Full AST-to-Tether-IR lowering (control flow, closures, mutable slots,
  ownership ops), optimization passes, machine IR, instruction selection,
  register allocation, native object emission, debug info.
- Cross-compilation and stable C ABI interoperability.

## Source layout

```
src/
  token.rs       Token enum + Spanned wrapper (line/col)
  lexer.rs       Hand-written single-pass lexer
  ast.rs         Expr, Stmt, Block, Program
  parser.rs      Pratt parser, precedence ladder in enum Prec
  value.rs       Runtime values, ownership slots, environments
  ownership.rs   Static ownership/borrow analysis pass
  interp.rs      Tree-walking reference interpreter (+ interp/)
  compiler.rs    AST to bytecode compiler (+ compiler/)
  bytecode.rs    Bytecode instruction/chunk/function types (+ bytecode/)
  vm.rs          Bytecode VM (default backend)
  ir/            Tether IR model, lowering, verifier, renderer
  scheduler/     Cooperative async tasks: spawn, join, select
  modules/       File-relative import graph loading and namespace lowering
  package/       Local manifest discovery, validation, scaffolding
  capability.rs  Capability trait/object model
  http*.rs       HTTP built-ins, client, and dynamic-handler server
  http_static/   Native cached static-file HTTP server
  https_server.rs Optional TLS server (feature: openssl-tls)
  database/      Database capability
  postgres/      Native PostgreSQL wire-protocol client
  json.rs        In-tree JSON parser/encoder
  system.rs      fs/process/env/path/time/hash/base64/url tools
  smtp.rs        SMTP support
  template.rs    Tera-compatible rendering (feature: tera)
  browser*.rs    Experimental HTML/CSS/layout/render + JS DOM host
  js*.rs         Dependency-free JavaScript engine surface
  lsp.rs         LSP stdio server (diagnostics only; see above)
  plugin.rs      Rust host plugin API
  actix_web.rs   Actix Web plugin (feature: actix-web)
  lib.rs         Library surface for embedding
  main.rs        CLI entry point
examples/
  ~97 .tether programs, including hello, fib, closures, ownership,
  use_after_move, async_basic, modules, static_site_server, https_server
tests/
  ~190 integration/regression test files
editor/vscode/
  VSCode grammar and language client (completions, hovers, navigation)
docs/
  standard tools, async runtime, capability APIs, CodeTether integration
```

`src/` is deeply modularized to honor the 50-line file limit: most top-level
`foo.rs` files have a sibling `foo/` directory holding the split-out concerns.

## Language design (LOAD-BEARING — do not change without explicit approval)

- **Dynamic typing.** No type annotations anywhere. Types are runtime tags.
- **Checked ownership.** Values carry a live/moved state. `move x` transfers;
  plain `x` borrows. Scalars (int, float, bool, nil) are Copy; everything else is
  genuinely owned. Enforcement is layered: a static pass rejects the lexically
  obvious violations up front, and the runtime is the backstop. Both must agree
  on semantics.
- **Expression-oriented blocks.** Last expression without `;` is the block's
  value. `if`, `while`, `{}` are all expressions.
- **Rust-like syntax.** Braces, `fn`, `let`, `let mut`, `&`, `&mut`, `move`.
- **Errors:** `panic` for bugs, `Result<T, E>` + `?` for recoverable failures.
- **Target use case:** agent / AI workloads. Async, HTTP, JSON, subprocess, and
  filesystem access are already first-class and in-tree. Channels remain a
  stdlib gap. Capability grants, not ambient access, are how scripts reach the
  outside world.

## Your working principles

1. **Small, focused PRs.** One concern per change. If a task needs multiple
   concerns, split it and pick one.
2. **Test everything you add.** Before claiming a feature works, add an
   example in `examples/` that exercises it AND unit tests in the relevant
   module. No feature lands without both.
3. **Reference interpreter first, optimizations later.** If a change would
   make the tree-walking interpreter harder to read, push it to the bytecode
   VM work instead. The tree-walker's job is clarity.
4. **Preserve running examples.** `cargo build --release` must succeed and the
   examples must still produce their documented output. Regressions are not
   acceptable; run the examples your change can plausibly affect, and the full
   test suite, after every change.
5. **Error messages matter.** Every error path must name the thing that went
   wrong (variable name, type name, source location). "Error" is not an error
   message.
6. **Zero new dependencies without justification.** The core build still has no
   required dependencies; JSON, HTTP, the async scheduler, and the JS/browser
   track were all written in-tree rather than pulled in. Tokio was considered and
   deliberately not adopted. Adding a crate is a design decision: justify it in
   the commit message and put it behind an opt-in feature flag, the way
   `actix-web`, `openssl-tls`, `native-window`, and `tera` are.
7. **Ask before renaming or restructuring.** The file layout and public type
   names are load-bearing for anyone reading the code. If you think something
   needs to move, raise it and get approval first.

## Prioritized task queue

Work these in order unless instructed otherwise. Each is one PR.

**Already shipped — do not re-open as new work:** integration + unit test
suites, CI, `LICENSE-MIT`, `CONTRIBUTING.md`, `for x in iterable`, `Result` +
`?`, string interpolation, static ownership analysis, the agent-facing stdlib
(HTTP/JSON/process/fs/env, all dependency-free and in-tree), the bytecode
compiler and VM, and the cooperative async scheduler (`spawn`/`join`/`select`,
implemented without Tokio).

### P0 — Correctness and honesty of the record

1. Keep `AGENTS.md`, `README.md`, and `CHANGELOG.md` consistent with the binary.
   When a claim and the code disagree, run the binary, then fix the docs.
2. Reconcile the `README.md` "What is not done yet" list against reality. It
   still lists shared-vs-mutable borrow conflict detection as absent even though
   `src/ownership.rs` rejects it statically; the accurate gap is *runtime*
   enforcement.

### P1 — Language completeness

3. **Runtime `&mut` exclusivity enforcement.** Add borrow counters to heap
   values (list, map, str) so dynamically-created aliases are caught, not just
   lexically obvious ones. Mutable borrow requires zero other borrows; shared
   borrow requires zero mutable borrows. Panic with a message naming the
   binding. The static pass in `src/ownership.rs` stays as the fast path.

### P2 — Tooling parity

4. **Promote LSP capabilities into `src/lsp.rs`.** Move completions, hover, and
   go-to-definition from `editor/vscode/lib/*.js` into the stdio server so
   Neovim, Helix, and Zed get them. Advertise the providers in the initialize
   result. Keep the VSCode client as a thin shell.
5. Exact diagnostic spans (currently line/col approximations).
6. Formatter (`tetherscript fmt`).

### P3 — Runtime performance

7. Inline caches for method lookup; NaN-boxing for `Value` if it does not hurt
   readability. Keep the interpreter as the reference semantics oracle and keep
   the suite runnable against either backend.
8. Full AST-to-Tether-IR lowering: control flow, closures, mutable slots, and
   ownership operations. Then an optimization pass framework and machine IR.

### P4 — Native backend and interop

9. Instruction selection, register allocation, native object emission, debug
   info, cross-compilation, and a stable C ABI. Major design discussion — stop
   and ask before starting.

### P5 — Browser and JS conformance

10. Continue the in-tree browser/JS parity track with explicit conformance
    tracking. Never delegate to an external browser engine or remote-control
    driver; tests assert that those are rejected.

## Working against CodeTether

You have access to the tetherscript codebase. When you pick up a task:

1. Re-read this prompt and the README to refresh context.
2. `cargo build --release` and confirm the baseline is clean before starting any
   change. Note that many of the ~97 examples now need capability grants
   (`--grant-fs`, `--grant-http`, `--grant-provider`) or bind ports, so a blind
   `for f in examples/*.tether` loop will report false failures. Prefer
   `cargo test` plus targeted runs of the examples your change actually touches.
3. Work the task. Keep the change surgical.
4. Before declaring done:
   - `cargo fmt`
   - `cargo clippy` — address all warnings
   - `cargo test` — all tests pass
   - `cargo test --doc` — doc examples still compile and run
   - `./check_file_limits.sh` — changed `src/**/*.rs` files stay within 50
     effective lines
   - Re-run the examples related to your change; they still produce their
     documented output
   - If you added a feature, add an example demonstrating it, and a test
     locking in its expected behavior
5. Commit with a message in the form:
   ```
   <area>: <one-line summary>

   <why, not what — the diff shows the what>

   Closes #<issue> (if applicable)
   ```
   Example: `parser: treat fn-followed-by-paren as anon fn expression`

## Stop conditions

Stop and surface a question to the human if:

- The task as stated is ambiguous or underspecified
- You find yourself about to modify a load-bearing design decision
  (dynamic typing, ownership model, expression-oriented blocks, syntax
  family)
- A feature seems to require a new dependency and you're not sure it's
  justified
- You discover a bug in existing code that's broader than the current task
- You're about to do something that would break a running example and
  don't see an obvious way around it

Do not speculate. Do not drift. Do not refactor opportunistically. One
change, tested, committed, reviewed. Then the next one.

## First action

Unless instructed otherwise: start with **P0** — confirm the documented state
matches the binary, and fix whichever side is wrong. The safety net (tests, CI)
already exists, so trust it but verify claims against a real run before building
a plan on top of them.


## Rustdoc & Documentation Standards

> **This is an open-source project.** Every public type, function, and module
> must be documented well enough that a junior developer can use it without
> reading the implementation. When in doubt, over-document.

> **Note on the examples below.** The illustrative snippets in this section were
> imported from the CodeTether agent codebase and still say
> `use codetether_agent::...`. Treat them as *formatting* patterns only. In this
> repository the crate is `tetherscript`, so write `use tetherscript::...`.

### Running Doc Tests

```bash
# Run ONLY doc tests (fast, catches broken examples)
cargo test --doc

# Run doc tests for a single module
cargo test --doc session

# Generate HTML docs and open in browser
cargo doc --open --no-deps
```

### Doc Comment Cheat Sheet

Rust doc comments use `///` for items and `//!` for module-level docs.

```rust
//! This is a module-level doc comment.
//!
//! It appears at the top of a file (usually `mod.rs` or `lib.rs`)
//! and describes what the entire module is for.

/// A single-line doc comment for an item below it.
///
/// A longer description goes here. You can use **bold**, *italic*,
/// and [`links to other types`](crate::session::Session).
///
/// # Arguments
///
/// * `name` — Description of the parameter.
///
/// # Returns
///
/// What the function returns and when it errors.
///
/// # Examples
///
/// ```rust
/// let result = 2 + 2;
/// assert_eq!(result, 4);
/// ```
pub fn my_function(name: &str) -> String {
    format!("Hello, {name}")
}
```

### Runnable vs Non-Runnable Examples

Rust has **four** doc example modes. Use the right one:

| Annotation | Compiles? | Runs? | Use When |
|---|---|---|---|
| ` ```rust ` or ` ``` ` | Yes | Yes | **Default. Pure logic, no I/O.** |
| ` ```rust,no_run ` | Yes | No | Compiles but needs network/files at runtime. |
| ` ```rust,ignore ` | No | No | Pseudocode or needs external context. |
| ` ```text ` | No | No | Output examples, diagrams, CLI output. |

**Rule: Prefer runnable (` ``` `) whenever possible.** If the example can't compile
without the rest of the crate, use `no_run`. Only use `ignore` as a last resort.

### Writing Runnable Doc Examples

Runnable examples are real Rust code that `cargo test --doc` compiles and executes.
They act as both documentation AND tests — if the example breaks, CI catches it.

#### Pattern 1: Simple function (fully runnable)

```rust
/// Truncate a string to `max_len` bytes, appending "..." if truncated.
///
/// # Examples
///
/// ```rust
/// use codetether_agent::tui::truncate_str;
///
/// assert_eq!(truncate_str("hello", 10), "hello");
/// assert_eq!(truncate_str("hello world", 8), "hello...");
/// ```
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let boundary = s.floor_char_boundary(max_len.saturating_sub(3));
        format!("{}...", &s[..boundary])
    }
}
```

#### Pattern 2: Struct with builder (fully runnable)

```rust
/// Result from executing a tool.
///
/// # Examples
///
/// ```rust
/// use codetether_agent::tool::ToolResult;
///
/// // Success case
/// let ok = ToolResult::success("file written");
/// assert!(ok.success);
/// assert_eq!(ok.output, "file written");
///
/// // Error case
/// let err = ToolResult::error("permission denied");
/// assert!(!err.success);
/// ```
pub struct ToolResult {
    pub output: String,
    pub success: bool,
}
```

#### Pattern 3: Async function (no_run — needs tokio runtime)

```rust
/// Load a session from disk by its UUID.
///
/// # Examples
///
/// ```rust,no_run
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// use codetether_agent::session::Session;
/// use std::path::Path;
///
/// let session = Session::load(Path::new("/tmp/sessions"), "abc-123")
///     .await
///     .expect("session should exist");
/// println!("Loaded {} messages", session.messages.len());
/// # });
/// ```
```

#### Pattern 4: Error handling (fully runnable)

```rust
/// Parse a tool call ID from a string.
///
/// # Errors
///
/// Returns `Err` if the string is empty or not valid UTF-8.
///
/// # Examples
///
/// ```rust
/// fn parse_id(s: &str) -> Result<String, String> {
///     if s.is_empty() {
///         return Err("ID cannot be empty".into());
///     }
///     Ok(s.to_uppercase())
/// }
///
/// assert_eq!(parse_id("abc").unwrap(), "ABC");
/// assert!(parse_id("").is_err());
/// ```
```

#### Pattern 5: Enum with match (fully runnable)

```rust
/// Outcome of an audited action.
///
/// # Examples
///
/// ```rust
/// use codetether_agent::audit::AuditOutcome;
///
/// let outcome = AuditOutcome::Success;
/// match outcome {
///     AuditOutcome::Success => println!("action succeeded"),
///     AuditOutcome::Failure => println!("action failed"),
///     AuditOutcome::Denied  => println!("action denied by policy"),
/// }
/// ```
```

### Hidden Lines in Doc Examples

Use `# ` (hash + space) to hide boilerplate lines. They still compile but
don't show in the rendered docs:

```rust
/// # Examples
///
/// ```rust
/// # use std::collections::HashMap;
/// # fn main() {
/// let mut map = HashMap::new();
/// map.insert("key", 42);
/// assert_eq!(map["key"], 42);
/// # }
/// ```
```

The user sees:
```rust
let mut map = HashMap::new();
map.insert("key", 42);
assert_eq!(map["key"], 42);
```

But `cargo test --doc` compiles the full version with imports and `fn main()`.

### Required Doc Sections

Every public item **must** have at minimum:

| Item Type | Required Sections |
|---|---|
| Module (`//!`) | Purpose, key types, usage overview |
| Struct | Purpose, `# Examples` with construction |
| Enum | Purpose, variants list, `# Examples` with match |
| Function | Purpose, `# Arguments`, `# Returns`, `# Examples` |
| Trait | Purpose, `# Implementors` or `# Examples` |
| Method | One-line summary + `# Examples` if non-obvious |

### When to Use `# Errors` and `# Panics`

```rust
/// # Errors
///
/// Returns [`anyhow::Error`] if:
/// - The session file does not exist
/// - The JSON is malformed
///
/// # Panics
///
/// Panics if `max_retries` is zero (this is a programming error).
```

**Rule:** Document `# Errors` for every function returning `Result`.
Document `# Panics` for every function that can panic.

### Linking to Other Types

Use intra-doc links so docs stay valid even if modules move:

```rust
/// Sends a message through the [`Session`] and records it
/// in the [`AuditLog`](crate::audit::AuditLog).
///
/// See also: [`ToolResult::success`]
```

### Module-Level Docs

Every `mod.rs` must start with `//!` docs:

```rust
//! # Session Management
//!
//! This module handles conversation persistence, message history,
//! and session lifecycle (create, load, save, list, delete).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! use codetether_agent::session::Session;
//! use std::path::Path;
//!
//! // Create a new session
//! let mut session = Session::new(Path::new("./sessions"));
//! session.add_user_message("Hello!");
//! session.save().await.unwrap();
//! # });
//! ```
//!
//! ## Architecture
//!
//! Sessions are stored as JSON files in the sessions directory.
//! Each session has a UUID, a list of messages, and metadata.
```

### CI Enforcement

Doc tests run in CI alongside unit tests. A broken doc example **blocks the PR**.

```bash
# This is what CI runs:
cargo test --doc          # All doc examples must pass
cargo doc --no-deps 2>&1  # No rustdoc warnings allowed
```


!Important we have formatting rules we are trying to implement, SRP Modular cohesion and 50 line file limits
## Hard Code Quality Rules

### **Modular Cohesion & Single Responsibility Principle (SRP)**
- **NEVER** mix concerns in a single file or function
- **EACH** module/file/function must have ONE clear responsibility
- **WHEN** a file handles multiple concerns, immediately refactor into separate modules
- **ALL** controllers must only handle HTTP concerns (request/response parsing)
- **ALL** business logic must be in separate model/service layers
- **ALL** database operations must be in dedicated repository/query modules

### **50-Line File Limit**
- **STRICT** 50-line maximum per file (excluding comments and blank lines)
- **WHEN** a file exceeds 50 lines, **MUST** split into smaller modules
- **IF** you're at 45+ lines, proactively refactor before hitting the limit
- **FILES** should be focused: one struct, one function group, or one concern
- **ENFORCEMENT** runs globally for changed `src/**/*.rs` files via `./check_file_limits.sh`
- **OVERSIZED LEGACY FILES** are grandfathered only as a ratchet: do not add lines before splitting them

### **Type Safety Enforcement**

This repository is Rust-first. The rules below were written for a TypeScript
codebase; apply the Rust equivalents:

- **NEVER** reach for a dynamic escape hatch where a real type belongs. In Rust
  that means no gratuitous `Box<dyn Any>` / stringly-typed values.
- **ALWAYS** give public functions explicit parameter and return types.
- **PREFER** inference for obvious locals; annotate when the type is load-bearing.
- The `editor/vscode/` client is JavaScript, so the TypeScript guidance applies
  there.

Original TypeScript wording, retained for the VSCode client:

- **NEVER** use `any` type - if the project maintainer sees `any`, they will assume you are a bad developer and will be forced to fix it without asking
- **ALWAYS** define explicit types for function parameters and return values
- **USE** TypeScript strict mode everywhere
- **PREFER** type inference (`const x = ...`) only when the type is obvious

### **Code Review Expectations**
These are **hard rules**, not suggestions. Violations will be rejected in code review.



!Important removing artifacts that validate your claims shows as obfuscation and can be intetpreted as hiding and lying