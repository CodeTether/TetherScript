# Agent TUI Scripts

tetherscript does not ship a built-in agent. It ships the language and host
primitives needed to write one in `.tether`.

The terminal surface is deliberately small and dependency-free:

- `tui_size()` returns `{ rows, cols }`, using terminal environment hints.
- `tui_enter()` and `tui_leave()` return alternate-screen/cursor lifecycle
  control strings for scripts that want to own the terminal canvas.
- `tui_render(view)` returns an ANSI-safe text frame for tests or logs.
- `tui_present(view)` clears and redraws the frame.
- `tui_read_event(prompt)` reads one line and returns `Ok({ type, text })`.
- `tui_read_key()` reads one key-sized stdin event and returns
  `Ok({ type, key, text, ctrl, alt, shift })`.
- `tui_style_open(style)`, `tui_style_reset()`, and `tui_span_render(span)`
  expose ANSI styling for maps with `fg`, `bg`, `bold`, `dim`, `underline`,
  and `inverse`.
- `tui_clear()`, `tui_cursor(visible)`, `tui_alt_screen(enabled)`, and
  `tui_move_to(row, col)` return raw ANSI control strings.

A view is a map with `title`, `status`, `width`, `height`, and `items`. Each
item may be a string or a map with `kind`, `name`, `text`, and optional style
fields.

Agent behavior stays in script. The reference example is a real stdio TUI by
default:

```text
read line from stdin -> update script state -> call provider/tool capability -> redraw TUI
```

Use `provider.chat(...)` for model calls when the host grants `--grant-provider`
or `--grant-provider-vault`.

For a CodeTether-like local run, use:

```bash
tetherscript run --access-mode full examples/agent_tui.tether
```

Inside the TUI, type prompts directly. Tool calls are sent to the model by
default. Manual tool checks are available with `/tool cwd`, `/tool ls <path>`,
`/tool read <path>`, and `/tool run <command>`.

If no `provider` capability is granted, prompts stay inside the TUI and render
a provider-missing message instead of crashing the process.

For a host that needs JSON-RPC over stdio, set explicit RPC mode:

```bash
TETHERSCRIPT_AGENT_MODE=rpc tetherscript run --access-mode full examples/agent_tui.tether
```

Then send newline-delimited JSON-RPC on stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"agent/message","params":{"prompt":"hi"}}
```

The script exposes `initialize`, `tools/list`, `tools/call`, and
`agent/message`. Its built-in tools are `cwd`, `ls`, `read`, `write`, and
`run`. Stdout is protocol-only so an external agent can parse it safely.
