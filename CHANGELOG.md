# Changelog

All notable changes to this project will be documented in this file.

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
