# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0-alpha.11] - 2026-05-11

### Added

- Added integration tests that run the core example programs and verify stdout against checked-in golden files.
- Added regression coverage for the use-after-move example's expected ownership error.

## [0.1.0-alpha.8] - 2026-05-02

### Added

- Dependency-free JavaScript interpreter (js module) with globals, functions, control flow, arrays, and js_eval.
- Browser JavaScript host bindings (browser_js module) exposing window/document, DOM mutation/querying, events, deterministic timers, and Storage APIs.
- Expanded browser subsystem with richer CSS selector parsing/cascade, structured snapshot and display-list output helpers.
- tetherscript js CLI subcommand for running JavaScript.
- Test coverage for img src attribute in display commands.

### Fixed

- EVENT_REGISTRY thread_local now cleared per evaluation to prevent memory leaks.
- <img src> DOM attribute carried into LayoutBox.styles so DisplayCommand::Image.src is non-empty.

## Unreleased

### Added

- Expanded experimental browser runtime built-ins with CSS rule introspection, computed styles, query selection, text extraction, page snapshots, framework-root/resource discovery, and structured display-list output.

## [0.1.0-alpha.6] - 2026-05-01

### Added

- Added first-class `Bytes` support across the language pipeline:
  - `Value::Bytes` runtime representation.
  - `b"..."` byte-string literals with `\xNN` escapes.
  - `bytes(...)` builtin for strings, byte lists, and bytes cloning.
  - Bytes indexing, index assignment, iteration, equality, truthiness, and display formatting.
  - Bytes methods: `len`, `push`, `pop`, `decode_utf8`, `to_string`, and `hex`.
- Added static ownership analysis in `src/ownership.rs`.
- Added `tetherscript check <file>` for parse plus statically-resolvable ownership checks.
- Added pre-execution ownership analysis to `tetherscript run`.
- Added VM instruction-budget enforcement for bytecode execution.
- Added `VM::builder()`, `VmBuilder`, and `tetherscript::Vm` re-export for embedders.
- Added tests covering bytes behavior in both interpreter and VM, plus bytes JSON encoding.

### Changed

- `tetherscript run <file>` now uses the bytecode VM by default.
- Added `--interp` / `--tree-walk` to run with the tree-walking interpreter for debugging.
- Normalized prerelease versioning from `0.0.1-alpha-0.5` to `0.1.0-alpha.6`.
- Completed crate metadata for publishing, including repository, readme, keywords, and categories.
- JSON encoding now represents bytes as arrays of integers without an intermediate `Vec<Value>` allocation.
- `bytes.hex()` now avoids per-byte temporary string allocations.

### Fixed

- Fixed VM byte literal semantics so mutable byte constants are deep-cloned on load and do not share buffers across evaluations.
- Fixed duplicate ownership diagnostics for borrow bindings.
- Fixed `fs.read` binary fallback to avoid cloning the entire file buffer when UTF-8 decoding fails.

## [0.1.0-alpha.5] - 2026-05-01

### Changed

- Initial alpha-stable feature publication. Superseded by `0.1.0-alpha.6` with PR review fixes.
