# tetherscript for VSCode

Language support for [tetherscript](https://github.com/tetherscript-Rs/tetherscript):
syntax highlighting plus live diagnostics from the built-in LSP server.

## What it does

- Syntax highlighting for `.tether` files
- Highlighting for keywords, functions, methods, fields, built-ins, `Ok`/`Err`,
  byte strings, regular strings, and string interpolation
- Lex and parse errors reported as red squigglies in real time
- Snippets for functions, loops, `Result` flow, maps, filesystem reads,
  provider calls, and TUI views
- `tetherscript: Run File` command for the active `.tether` file
- `tetherscript: Show Tokens`, `Show AST`, and `Show Bytecode` commands
- `tetherscript: Run Agent TUI` command for `examples/agent_tui.tether`
- Completion suggestions for keywords, constants, built-ins, and methods
- Hover docs for common built-ins, keywords, constants, and methods
- Document outline for `fn` declarations
- Same-file go-to-definition for function calls
- Run CodeLens on `fn main`
- `tetherscript: Kill Language Server`, `Start Language Server`, and
  `Restart Language Server` commands
- Bracket matching, auto-close, and comment toggling

## Install from source

1. Build the `tetherscript` binary and make sure it is on your `PATH`:

   ```bash
   cargo install --path .
   # or: cargo build --release && cp target/release/tetherscript ~/.local/bin/
   ```

2. Install the extension dependencies:

   ```bash
   cd editor/vscode
   npm install
   ```

3. Open `editor/vscode` in VSCode and press `F5` to launch an Extension
   Development Host. Open a `.tether` file in the new window.

   To package and install for daily use:

   ```bash
   npx vsce package
   code --install-extension tetherscript-0.0.9.vsix
   ```

## Settings

- `tetherscript.serverPath`: path to the `tetherscript` binary.
- `tetherscript.runArgs`: extra arguments for `tetherscript: Run File`, such as
  `--access-mode full`.
- `tetherscript.trace.server`: LSP trace mode (`off`, `messages`, `verbose`).

## Missing

The server currently reports lex and parse diagnostics only. The extension has
same-file function navigation; rename, cross-file navigation, and exact semantic
tokens require source spans on AST nodes, which are on the roadmap.
