# Agent TUI Scripts

tetherscript does not ship a built-in agent. It ships the language and host
primitives needed to write one in `.tether`.

The terminal surface is deliberately small and dependency-free:

- `tui_size()` returns `{ rows, cols }`, using terminal environment hints.
- `tui_render(view)` returns an ANSI-safe text frame for tests or logs.
- `tui_present(view)` clears and redraws the frame.
- `tui_read_event(prompt)` reads one line and returns `Ok({ type, text })`.
- `tui_clear()`, `tui_cursor(visible)`, `tui_alt_screen(enabled)`, and
  `tui_move_to(row, col)` return raw ANSI control strings.

A view is a map with `title`, `status`, `width`, `height`, and `items`. Each
item may be a string or a map with `kind`, `name`, and `text`.

Agent behavior stays in script. The reference example speaks JSON-RPC over
stdin/stdout and renders its status frame to stderr:

```text
read JSON-RPC -> update script state -> call provider/tool capability -> write JSON-RPC
```

Use `provider.chat(...)` for model calls when the host grants `--grant-provider`
or `--grant-provider-vault`.

For a CodeTether-like local run, use:

```bash
tetherscript run --access-mode full examples/agent_tui.tether
```

Send newline-delimited JSON-RPC on stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"agent/message","params":{"prompt":"hi"}}
```

The script exposes `initialize`, `tools/list`, `tools/call`, and
`agent/message`. Its built-in tools are `cwd`, `ls`, `read`, `write`, and
`run`. Stdout is protocol-only so an external agent can parse it safely.
