# TetherScript for VSCode

Language support for [TetherScript](https://github.com/TetherScript-Rs/tetherscript) — syntax highlighting plus live diagnostics from the built-in LSP server.

## What it does

- Syntax highlighting for `.tether` files and legacy `.tether` files
- Lex and parse errors reported as red squigglies in real time
- Bracket matching, auto-close, comment toggling

## Install (from source, for now)

1. Build the `tetherscript` binary and make sure it's on your `PATH`:

   ```bash
   cargo install --path .
   # or: cargo build --release && cp target/release/tetherscript ~/.local/bin/
   ```

2. Install the extension dependencies:

   ```bash
   cd editor/vscode
   npm install
   ```

3. Open `editor/vscode` in VSCode and press `F5` to launch an Extension Development Host. Open a `.tether` or `.tether` file in the new window.

   To package and install for daily use:

   ```bash
   npx vsce package
   code --install-extension tetherscript-0.0.4.vsix
   ```

## Settings

- `tetherscript.serverPath` — path to the `tetherscript` binary (default: `tetherscript`, assumed to be on PATH).
- `tetherscript.trace.server` — trace LSP traffic for debugging (`off` / `messages` / `verbose`).

## What's missing

The server currently reports lex and parse diagnostics only. Hover, go-to-definition, and completion require source spans on AST nodes, which are on the roadmap.
