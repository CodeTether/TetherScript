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

A view is a backend-neutral map with `title`, `status`, `width`, `height`, and
`items`. Each item may be a string or a map with `kind`, `name`, `text`, and
optional style fields. Pass the same map to `tui_present(view)` for a terminal,
or to `surface.render_view(view)` and `surface.present()` for a native window.
See [`examples/dual_ui.tether`](../examples/dual_ui.tether).

Agent behavior stays in script. The reference example is a real stdio TUI by
default:

```text
read line from stdin -> update script state -> call provider/tool capability -> redraw TUI
```

Use `provider.chat(...)` for model calls when the script declares agent
authority or when the host grants `--grant-provider` or `--grant-provider-vault`.

For a CodeTether-like local run, use:

```bash
tetherscript run examples/agent_tui.tether
```

The example starts with `// tetherscript: authority agent`, so the runner grants
the same local agent authority that `--access-mode full` provides: filesystem
authority to the current directory plus provider loading from Vault or Windows
environment variables.

It also declares `// tetherscript: hot-reload`. After each prompt turn, the TUI
compares its loaded source with the file on disk. If an agent edits
`examples/agent_tui.tether`, the script exits through a safe turn boundary,
writes `.tetherscript/reload`, and the runner reloads the updated source in the
same terminal process. `/quit` still exits normally because it does not write the
reload marker.

Inside the TUI, type prompts directly. Tool calls are sent to the model by
default. Manual tool checks are available with `/tool cwd`, `/tool ls <path>`,
`/tool read <path>`, and `/tool run <command>`.

For persistent native browser tools, start the native host in one terminal:

```bash
tetherscript-browser-host 127.0.0.1:41707
```

Then grant it to the TUI in another terminal, narrowing origins in production:

```bash
tetherscript run \
  --grant-browser http://127.0.0.1:41707/browser \
  --browser-origin http://localhost:5173 \
  --browser-scope all \
  examples/agent_tui.tether
```

The model-visible browser tools are `browser_goto`, `browser_click`,
`browser_text`, and `browser_snapshot`. They share one native page for the
duration of the host process.

If no `provider` capability is granted, prompts stay inside the TUI and render
a provider-missing message instead of crashing the process.

For a host that needs JSON-RPC over stdio, set explicit RPC mode:

```bash
TETHERSCRIPT_AGENT_MODE=rpc tetherscript run examples/agent_tui.tether
```

Then send newline-delimited JSON-RPC on stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"agent/message","params":{"prompt":"hi"}}
```

The script exposes `initialize`, `tools/list`, `tools/call`, and
`agent/message`. Its built-in tools include workspace, shell, native browser,
and JavaScript operations. Stdout is protocol-only so an external agent can
parse it safely.
