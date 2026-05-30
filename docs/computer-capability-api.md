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
