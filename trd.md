# Tetherscript — Product Requirements Document

**Version:** 0.1 (Alpha)
**Owner:** Riley Seaburg
**Status:** Draft
**Last updated:** 2026-05-01

---

## TL;DR

Tetherscript is an embeddable, dynamically-typed scripting language with Rust-style ownership semantics, implemented as a bytecode VM in Rust. Its primary purpose is to provide a safe, sandboxable scripting layer for Rust applications — particularly agent runtimes that need to execute semi-trusted, often LLM-generated code with strong isolation guarantees at the language level, not just the OS level.

The reference host is CodeTether. Secondary hosts are any Rust application that today reaches for Lua (mlua), Rhai, or embedded QuickJS-on-WASM and finds the safety or concurrency story lacking.

Tetherscript is **not** a general-purpose application language and **not** a Rust replacement. It **does** aim, as a long-horizon product direction, to support arbitrary web-page rendering and browser APIs through an ownership-aware browser/runtime stack. The near-term alpha remains focused on the embeddable VM and host control plane, but the browser target is now an explicit product goal rather than a non-goal.

---

## Problem

Embedded scripting in Rust applications is a solved problem only in the boring case.

- **Lua (mlua):** mature, fast, but GC'd; dynamically typed with no safety story beyond memory; semantics are alien to Rust hosts; capability handling is host-side hand-rolling.
- **Rhai:** Rust-shaped but statically typed and without ownership semantics. Loses the ergonomic dynamism that makes scripting useful.
- **QuickJS / Boa on WASM:** isolation via the WASM boundary works, but the per-VM cost is heavy, the language has no ownership model, and the WASM tax is paid every call.

The specific unfilled gap: a **dynamic** language with **Rust-style ownership** — moves, borrows, single-mutable-or-shared-immutable, scope-based lifetimes — does not exist in any production form. The dynamic + ownership combination is research-adjacent (Hylo/Val and Mojo are static) but has not been productized.

This matters because:

1. **Agent runtimes execute LLM-generated code.** That code is semi-trusted at best. Language-level ownership rules prevent whole classes of misbehavior — aliased mutation across task boundaries, capability leaks, use-after-revocation — before OS sandboxing has to catch them. Catching it at the language layer is cheaper and more precise.
2. **Rust hosts want a scripting story without becoming GC-aware.** mlua forces the host to think about Lua's GC. WASM forces serialization at every boundary. A native ownership-respecting interpreter avoids both.
3. **The combinatorics of "dynamic typing + ownership" are real engineering territory** — interesting enough to differentiate, narrow enough to ship.

---

## Goals

1. Be a credible Lua/Rhai alternative for Rust hosts that need stronger safety properties for extension scripting.
2. Provide ownership semantics that an LLM-generated script cannot violate without explicit, host-gated escape hatches.
3. Integrate cleanly with CodeTether as the reference host and primary dogfood target.
4. Ship as both an embedded library and a standalone CLI (`tetherscript run script.ts`), with the CLI serving development, debugging, and small operational scripting — not as the primary distribution channel.
5. Deliver a 1.0 with API stability sufficient for production use in CodeTether and at least one external host.
6. Establish a browser/runtime track capable of eventually rendering arbitrary web pages and exposing browser-compatible APIs under Tetherscript ownership and capability rules.

---

## Non-goals

These remain load-bearing, but the browser boundary has changed: Tetherscript is no longer explicitly excluding browser APIs or arbitrary page rendering. Instead, browser support is a major long-term product track layered on top of the safe embeddable VM.

1. **Not a short-term browser engine promise.** Tetherscript aims to render arbitrary web pages and implement browser APIs, but this is a multi-phase roadmap, not a v1 guarantee. DOM, HTML/CSS parsing, layout, paint/raster, async event loops, fetch streaming, WebGL, font shaping, animation frames, storage APIs, workers, accessibility, and web compatibility require browser-engine-class investment. The project will treat this as a staged runtime/platform effort rather than pretending it is a small scripting-language feature.
2. **Not a JS / Python / Ruby replacement.** No claim to replace general-purpose application languages. The browser goal means Tetherscript may need JS interop/compatibility at web boundaries, but the core language remains a safe, ownership-aware scripting/runtime language.
3. **Not a Rust replacement.** Tetherscript is a bytecode interpreter, not a JIT-compiled native path. Performance ceiling is "fast enough for scripting and browser control/orchestration," not "fast enough for compute kernels." Heavy lifting stays in Rust/native subsystems; Tetherscript orchestrates and enforces capability/ownership policy.
4. **Not multi-threaded in v1.** Single-threaded VM. Concurrency is achieved at the host level via one-VM-per-task with message passing. Browser-runtime async/concurrency primitives are a post-v1 platform track unless required earlier for prototype rendering.
5. **Not statically typed.** Dynamic typing is a deliberate choice for scripting ergonomics. Ownership is checked at compile-time where statically inferable, at runtime where not. Static type inference is an enhancement, not a v1 commitment.

---

## Target users and use cases

### Primary user

A Rust application developer who needs an extension language with stronger safety properties than Lua or Rhai provide, without adopting WASM. Initially: CodeTether maintainers and integrators. Eventually: any Rust shop with the same shape of problem (agent platforms, plugin systems, orchestration tooling).

### Primary use cases

1. **Sandboxed agent task execution (CodeTether).** An LLM emits a Tetherscript program. A CodeTether worker spins up a VM, registers host capabilities (tools, file handles, HTTP clients, MCP server handles) as borrowed or moved resources, runs the script, and tears down. Ownership rules prevent the script from aliasing capabilities across boundaries it shouldn't.
2. **Plugin scripting in Rust applications.** Hosts expose typed APIs; scripts call them with ownership-aware semantics that the host enforces on resource handles.
3. **Configuration and orchestration scripting.** A more expressive alternative to YAML+templating or Lua-in-config, where resource lifecycle matters.
4. **Browser/runtime scripting and rendering.** Long-term: a Tetherscript-powered browser/runtime stack that can render arbitrary web pages, expose DOM and Web APIs, and run web-facing logic with ownership-aware capabilities. Near-term: control-plane integration with host-provided rendering engines while Tetherscript subsystems mature.

### Anti-personas

- Data scientists wanting NumPy / pandas equivalents.
- Anyone expecting browser compatibility in the alpha VM without host-renderer support.
- Anyone who needs LuaJIT, V8, or browser-engine performance immediately.
- Anyone assuming that "browser APIs" means unconstrained ambient authority; Tetherscript browser APIs must remain capability-gated.

---

## Functional requirements

### Language

- Dynamic typing with primitives: `Int`, `Float`, `String`, `Bool`, `List`, `Map`, `Bytes`.
  - **`Bytes` is a first-class primitive, not `List<Int>`.** This is a correctness item, not a nice-to-have. Byte-oriented work (hashing, network buffers, binary protocols) is a hard requirement and the wrong representation makes it impossible to do efficiently. Fix before alpha-stable.
- Ownership: every binding owns its value; assignment moves by default; explicit `&` for shared borrow, `&mut` for exclusive borrow; the standard exclusion-or rule applies.
- Lifetimes: scope-based, no generic lifetime parameters in v1. Borrows cannot outlive the owner's scope.
- Functions, closures with explicit capture mode (`move`, `&`, `&mut`), recursion.
- Error handling: result-style return values. No exceptions.
- Module system: file-per-module, explicit imports, no global namespace pollution.

### Runtime

- Bytecode VM in Rust. Tree-walking AST acceptable for early alphas; bytecode required for alpha-stable.
- Single-threaded execution.
- Host-configurable instruction-count limit and memory limit per VM. Hard quota enforcement.
- Deterministic execution: no implicit clock, no implicit RNG, no implicit network or filesystem. All non-determinism enters via host-provided capabilities.
- Cold start under 50ms for a small script, excluding host setup.

### Host integration (Rust API)

- `tetherscript::Vm` builder pattern with capability registration.
- Typed value marshalling between Rust and Tetherscript via derive macro (target: alpha-stable; manual registration acceptable until then).
- Host can pass borrowed Rust references into the VM with non-escape checked at the FFI boundary.
- Host can revoke capabilities mid-execution; the VM raises ownership-violation errors gracefully without crashing.

### CLI

- `tetherscript run <file>`
- `tetherscript repl`
- `tetherscript check <file>` — parse plus ownership analysis where statically resolvable
- No package manager in v1. Module imports are file-relative. Defer package management until there is pull from real users.

---

## Non-functional requirements

| Property | Target |
|---|---|
| Throughput | Within 5x of Lua 5.4 on string and small-data-structure benchmarks. Do not benchmark against LuaJIT or V8. |
| Memory | VM baseline under 5MB. Host can cap per-VM memory. |
| Cold start | Under 50ms for trivial scripts. |
| Library footprint | Compiled release artifact under 2MB. |
| MSRV | Rust 1.75+ (subject to revision). |
| License | MIT. |
| Platforms | Linux x86_64 and aarch64 in v1. macOS and Windows best-effort. |

---

## Architecture

- **Frontend:** lexer → parser → AST.
- **Mid-end:** ownership analysis pass. Statically resolvable cases lower to no-op runtime checks; unresolved cases lower to bytecode runtime checks.
- **Bytecode:** stack-based VM, target ~80–120 opcodes.
- **Value representation:** decision pending. NaN-boxing is the candidate for compactness; tagged union is the baseline. Decide by alpha-stable.
- **Host bridge:** `extern fn` registration with derive-macro-generated wrappers (post-alpha).

---

## Milestones

| Milestone | Target | Definition of done |
|---|---|---|
| Alpha | Now (shipped 2026-05-01) | Tree-walking interpreter, basic ownership, no host bridge yet. 5 versions published, ~11k LOC. |
| Alpha-stable | Q2 2026 | Bytecode VM lands. Ownership analysis pass complete. `Bytes` primitive correct. Crate metadata complete (`repository`, `keywords`, `categories`). |
| Beta | Q3 2026 | Host bridge with derive macro. CodeTether integration shipped end-to-end. At least one external host smoke-tested. SemVer normalized. |
| 1.0 | Q4 2026 | API stability commitment. Documentation site. At least two production hosts (CodeTether plus one external). Naming decision finalized. |

---

## Success metrics

- CodeTether routes at least 50% of agent task execution through Tetherscript by beta.
- At least one external Rust project depends on `tetherscript` in production by 1.0.
- Crate downloads exceed 10k/month by 1.0.
- Zero known soundness holes in ownership rules at 1.0 — no script-level pattern that can alias a mutable capability without explicit `unsafe` and host opt-in.

---

## Risks and open questions

### Risks

1. **Dynamic + ownership is research-adjacent.** Edge cases in the dynamic-typing / ownership interaction may force breaking design changes. Mitigation: hold 1.0 until CodeTether dogfooding has shaken out the surface.
2. **Naming collision.** Tetherscript Inc. (controlmyjoystick.com, Windows HID virtual drivers, ~2013) occupies the surface namespace. No trademark conflict appears likely, but SEO and discoverability cost is real. Decision required before beta: accept the cost, or rename.
3. **Single-maintainer bus factor.** Mitigation: clear architecture, contributor-friendly docs, MIT license, recruit a second maintainer before 1.0.
4. **Niche too narrow.** If "Lua but with ownership" doesn't pull external users, the project stays a CodeTether internal tool. Acceptable outcome but caps ambition.
5. **Performance ceiling.** Bytecode VM tops out well below LuaJIT and V8. Document this as a characteristic, not a hidden weakness. Hosts that need JIT speeds are the wrong audience.

### Open questions

1. Value representation: NaN-boxing vs tagged union. Decide by alpha-stable.
2. Async story: cooperative async in v1, or host-level only? Default: host-level. Revisit at beta.
3. FFI macro aggressiveness: start conservative with manual registration, add derive at beta.
4. Standalone CLI scope creep: should it ever grow a package manager? Default no, defer until external pull.
5. Renaming decision deadline: beta cut.
6. SemVer: current `0.0.1-alpha-0.5` pattern sorts lexically in surprising ways (`0.0.1-alpha-0.10` < `0.0.1-alpha-0.5`). Move to `0.1.0-alpha.N` before alpha-stable, while there are zero downstream dependents.