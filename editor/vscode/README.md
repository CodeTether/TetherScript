# tetherscript for VSCode

Language support for [tetherscript](https://github.com/tetherscript-Rs/tetherscript):
syntax highlighting plus live diagnostics from the built-in LSP server.

## What it does

- Syntax highlighting for `.tether` files
- Highlighting for keywords, functions, methods, fields, built-ins, `Ok`/`Err`,
  byte strings, regular strings, and string interpolation
- Lex and parse errors reported as red squigglies in real time
- Module diagnostics for missing files, invalid or escaping paths, duplicate
  aliases, undeclared exports, import cycles, and access to non-exported members
- Snippets for functions, loops, `Result` flow, maps, filesystem reads,
  provider calls, TUI views, imports, exports, and exported functions
- `tetherscript: Run File` command for the active `.tether` file
- `tetherscript: Show Tokens`, `Show AST`, and `Show Bytecode` commands
- `tetherscript: Run Agent TUI` command for `examples/agent_tui.tether`
- Import-path completion from `.tether` files inside the current package
- Export-aware completion after an imported namespace, such as `math.`
- Auto-import completion for exported functions and values; accepting a suggestion
  inserts the namespace import and qualified call together
- Completion suggestions for all runtime tools, owned-resource factories and
  methods, keywords, constants, and `db.query`
- Hover docs for runtime tools, owned resources, language words, and SQL capabilities
- Hover summaries for imported namespaces and exported members
- Document outline for `fn` declarations
- Same-file and imported-export go-to-definition, plus clickable import paths
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

   Run the fast, dependency-free source-index tests:

   ```bash
   npm test
   ```

   Run the real VS Code extension-host suite (requires the `code` CLI):

   ```bash
   npm run test:integration
   ```

   The integration suite exercises the registered VS Code providers directly;
   it does not replace VS Code APIs with mocks.

   To package and install for daily use:

   ```bash
   npx vsce package
   code --install-extension tetherscript-0.0.10.vsix
   ```

## Settings

- `tetherscript.serverPath`: path to the `tetherscript` binary.
- `tetherscript.runArgs`: extra arguments for `tetherscript: Run File`, such as
  `--access-mode full`.
- `tetherscript.trace.server`: LSP trace mode (`off`, `messages`, `verbose`).

## Missing

The LSP server still owns lex and parse diagnostics. Module intelligence is
provided by the extension and follows the local package contract: file-relative
`.tether` imports, explicit exports, namespaced access, and package-root
containment. Rename and exact semantic tokens still require source spans on AST
nodes and remain on the roadmap.

Auto-import suggestions preserve namespace isolation. Selecting an exported
`add` function from `math.tether` inserts both `import "./math.tether" as math`
and `math.add(...)`; it never introduces an unsupported direct-symbol import.