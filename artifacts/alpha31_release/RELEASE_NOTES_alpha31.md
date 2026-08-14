# tetherscript v0.1.0-alpha.31

This release replaces the opt-in external native UI stack with a native
dashboard implemented inside tetherscript.

## Highlights

- Native dashboard layout, rendering, input, and agent state now live in-tree.
- The native agent uses tetherscript's software HTML/CSS rasterizer and bitmap
  text renderer while retaining the script-owned JSON-RPC backend.
- Native prompt editing accepts Unicode input, Enter submission, Backspace
  editing, and Escape-to-close window behavior.
- `eframe`, `egui`, `fontdue`, `iced-x86`, and `object` are removed from the
  crate manifest and lockfile.
- The default core remains dependency-free. The `native-window` feature keeps
  only the existing optional framebuffer transport.
- VM returns from inside `for` and `while` now discard the returning frame's
  loop operands instead of leaking them into the caller and corrupting a later
  call.

## Validation gates

Release validation covers formatting, Clippy with warnings denied, default and
feature test suites, rustdoc, package verification, local installation, and a
crate publish dry run. The native examples are syntax-checked in the release
profile; graphical smoke testing requires a host display.

## Install

```bash
cargo install tetherscript --version 0.1.0-alpha.31
```