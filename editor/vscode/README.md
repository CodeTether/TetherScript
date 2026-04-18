# Kiln for VSCode

Language support for [Kiln](https://github.com/Kiln-Rs/kiln) — syntax highlighting plus live diagnostics from the built-in LSP server.

## What it does

- Syntax highlighting for `.kl` files
- Lex and parse errors reported as red squigglies in real time
- Bracket matching, auto-close, comment toggling

## Install (from source, for now)

1. Build the `kiln` binary and make sure it's on your `PATH`:

   ```bash
   cargo install --path .
   # or: cargo build --release && cp target/release/kiln ~/.local/bin/
   ```

2. Install the extension dependencies:

   ```bash
   cd editor/vscode
   npm install
   ```

3. Open `editor/vscode` in VSCode and press `F5` to launch an Extension Development Host. Open a `.kl` file in the new window.

   To package and install for daily use:

   ```bash
   npx vsce package
   code --install-extension kiln-0.0.4.vsix
   ```

## Settings

- `kiln.serverPath` — path to the `kiln` binary (default: `kiln`, assumed to be on PATH).
- `kiln.trace.server` — trace LSP traffic for debugging (`off` / `messages` / `verbose`).

## What's missing

The server currently reports lex and parse diagnostics only. Hover, go-to-definition, and completion require source spans on AST nodes, which are on the roadmap.
