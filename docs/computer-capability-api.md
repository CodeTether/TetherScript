# Computer Capability API

TetherScript exposes the agent harness `computer_use` tool as a scriptable
capability named `computer`. The point is not to invent a second desktop API;
it is to let an agent enhance its existing screenshot, window, mouse, and
keyboard tool with reusable `.tether` automation.

The interpreter does not call Win32, macOS Accessibility, X11, or the
CodeTether developer tool directly. A host grants an authority that forwards
TetherScript calls to the agent/harness `computer_use` action surface.

Canonical mDNS service type for discovery:

```text
_codetether-computer._tcp.local.
```

## Granting

A harness grants the capability explicitly and origin-binds the authority:

```rust,ignore
host.grant(
    "computer",
    tetherscript::computer_cap::ComputerAuthority::new(
        "http://127.0.0.1:41707/computer-use/v1/rpc",
        tetherscript::computer_cap::ComputerAuthority::all_scopes(),
    ),
);
```

Hosts should map configuration fields as:

- `grant_computer` — endpoint for the local computer bridge
- `computer_scope` — allowed action scopes
- `computer_origin` — origin/audience allowed to use the grant

TetherScript forwards `computer_origin` to the bridge as
`X-TetherScript-Origin`, so the harness can reject cross-origin or stale grants
before touching privileged `computer_use` actions.

## Native Host Contract

For a method like `computer.snapshot()` TetherScript sends the same action name
used by CodeTether `computer_use`:

```http
POST /computer-use/v1/rpc
Content-Type: application/json

{"action":"snapshot"}
```

For `computer.click(params)`, pass a map:

```tether
let p = map()
p.x = 100
p.y = 200
computer.click(p)
```

This sends:

```json
{"action":"click","x":100,"y":200}
```

The bridge may return raw JSON, `{ "ok": true, "value": ... }`, or a
CodeTether tool result with `success` and `output`. Errors use `{ "ok": false,
"error": "..." }` or `success: false`.

## Scopes

Scopes are action-style and match the agent-facing contract:

- `computer.snapshot` — `snapshot`
- `computer.window_snapshot` — `window_snapshot`, window focus helpers
- `computer.click` — click, drag, mouse, and Blender frame helpers
- `computer.type` — `type_text`
- `computer.key` — `press_key`
- `computer.scroll` — `scroll`
- `computer.apps` — app/status/session helpers

`computer.narrow({ scopes: [...] })` can only remove scopes; it cannot add new
authority.

## Named logical cursors

Named cursors provide stable, independent window-relative coordinate state for nested desktops such as RDP and applications such as Blender. Moving a logical cursor does **not** move the OS pointer. The bridge is contacted only for click, drag, or window-snapshot actions.

```tether
let rdp = map()
rdp.name = "rdp"
rdp.hwnd = 42599730
rdp.x = 725
rdp.y = 432
rdp.client_area = true
computer.cursor_set(rdp)

let move = map()
move.name = "rdp"
move.dx = 0
move.dy = 31
computer.cursor_move(move) // local state only

let click = map()
click.name = "rdp"
computer.cursor_click(click) // dispatches the real click
```

Available methods:

| Method | Purpose | Bridge action / scope |
|---|---|---|
| `cursor_set({name, hwnd, x, y, client_area?})` | Create/reset local cursor state | none |
| `cursor_move({name, dx, dy})` | Move local state relatively | none |
| `cursor_state({name})` | Read local state | none |
| `cursor_click({name, button?})` | Click at stored coordinates | `click`/`right_click`; `computer.click` |
| `cursor_drag({name, dx, dy, button?, duration_ms?, steps?})` | Drag relatively and store endpoint | `drag`; `computer.click` |
| `cursor_snapshot({name})` | Snapshot the target window | `window_snapshot`; `computer.window_snapshot` |

Cursor registries are shared by narrowed child authorities, but dispatched actions still enforce the child's current scopes. The outgoing payload remains the existing computer bridge contract. See `examples/computer_named_cursors.tether` for an RDP/Blender example.

## Readiness

A host that serves the bridge should expose:

```http
GET /ready
```

Expected payload:

```json
{
  "ok": true,
  "capabilities": ["computer"],
  "mdns": "_codetether-computer._tcp.local.",
  "contract": "v1"
}
```
