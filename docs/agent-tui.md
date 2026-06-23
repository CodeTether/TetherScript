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

Agent behavior stays in script:

```text
read terminal event -> update script state -> call provider/tool capability -> render state
```

Use `provider.chat(...)` for model calls when the host grants `--grant-provider`
or `--grant-provider-vault`.

For a CodeTether-like local run, use:

```bash
tetherscript run --access-mode full examples/agent_tui.tether
```
Use standard tools such as `process_run`, `fs_read`, `fs_list`, and `cwd` for
local tool calls. The reference example is `examples/agent_tui.tether`.

For agent-driven tool calls over stdio, use `examples/stdio_mcp_tui.tether`.
That script is not a human chat prompt. It reads newline-delimited JSON-RPC on
stdin, writes protocol responses on stdout, and writes its TUI/status frame on
stderr so an external agent can safely parse stdout.
