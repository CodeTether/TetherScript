# Browser Capability API

TetherScript now has a live browser authority (`browser_cap::BrowserAuthority`) designed as the seam between scripts and real browser infrastructure. It does not emulate Chromium. The authority posts compact browserctl action envelopes to a browserctl-compatible HTTP bridge and records an in-process trace for audit and replay.

## Granting

CLI example:

```bash
tetherscript run --interp \
  --grant-browser http://127.0.0.1:41707/browser \
  --browser-origin http://localhost:5173 \
  --browser-scope all \
  examples/browser_agentic_debug.tether
```

Default scopes, when `--browser-scope` is omitted, are enough for Milestone 1: navigate, interact, DOM inspect, console inspect, network inspect, screenshot.

Supported scopes:

- `browser.navigate`
- `browser.interact`
- `browser.inspect.dom`
- `browser.inspect.network`
- `browser.inspect.console`
- `browser.inspect.storage`
- `browser.inspect.react`
- `browser.mutate.storage`
- `browser.replay.network`
- `browser.screenshot`
- `browser.visual`

Authorities can be narrowed with a map containing `origins`, `scopes`, `path_prefix`, `storage_scope`, and `human_approval`.

## Browserctl bridge contract

For a method like `browser.goto(url)`, TetherScript sends:

```http
POST /browser
Content-Type: application/json

{"action":"goto","url":"http://localhost:5173"}
```

The bridge may return either a raw JSON value or:

```json
{"ok": true, "value": {}}
```

Errors use:

```json
{"ok": false, "error": "selector not found"}
```

CodeTether-style tool results are also accepted. When a successful response has
`success: true` and string `output`, TetherScript parses `output` as JSON when
possible and otherwise returns it as a string.

## MVP methods

Navigation/interaction:
`goto`, `reload`, `back`, `forward`, `click`, `click_text`, `type`, `press`, `hover`, `focus`, `blur`, `scroll`, `wait_for_selector`, `wait_for_text`, `wait_for_url`, `wait_for_network_idle`.

The wait helpers are tetherscript convenience methods over browserctl's single
`wait` action:

- `wait_for_selector(selector, timeout_ms?)` sends `{"action":"wait","selector":...,"state":"visible","timeout_ms":...}`.
- `wait_for_text(text, timeout_ms?)` sends `{"action":"wait","text":...,"timeout_ms":...}`.
- `wait_for_url(url_substring, timeout_ms?)` sends `{"action":"wait","url_contains":...,"timeout_ms":...}`.

`wait_for_network_idle`, `compare_screenshots`, and `visual_diff` are reserved
language-level methods. The current browserctl backend rejects them before
network I/O until matching bridge actions exist.

Snapshots:
`screenshot`, `screenshot_element`, `dom_snapshot`, `page_snapshot`.

Diagnostics:
`console_logs`, `console_errors`, `unhandled_rejections`, `runtime_exceptions`, `source_mapped_stack_traces`.

Network:
`network_log`, `failed_requests`, `request`, `response`, `replay_request`, `wait_for_request`, `wait_for_response`.

Storage:
`cookies`, `local_storage`, `session_storage`, `indexed_db_summary`, `set_cookie`, `set_local_storage`, `clear_storage`.

React/framework hooks:
Use string method syntax for dotted method names, e.g. `browser."react.detect"()?`, `browser."react.component_for_selector"("#root")?`, plus `frameworks()` for Next/Vite/Redux/Zustand/React Query detection returned by the bridge.

Trace/export:
`trace`, `export_trace_json`, `export_har`, `agent_summary`, `minimal_reproduction_script`.

Raw browserctl:
`raw(action, params)` sends an explicit browserctl action envelope after the same
scope and origin checks as high-level methods. This is an FFI-style escape hatch;
the stable API remains the language-level browser methods above. Raw action
names are restricted to browserctl's advertised action enum.

## Agent assertions

Runtime assertion helpers return `Result` values suitable for `?` propagation:

- `assert_selector(browser, selector)`
- `assert_text(browser, text)`
- `assert_no_console_errors(browser)`
- `assert_no_failed_requests(browser)`
- `assert_visible(browser, selector)`
- `assert_enabled(browser, selector)`
- `assert_route(browser, path_or_url_substring)`
- `assert_screenshot_matches(browser, name)`
- `assert_react_component(browser, name)`

## Page snapshot schema target

The bridge should make `page_snapshot()` return a compact map with:

- url, title, viewport, scroll_position, focused_element, selected_element, visible_text
- dom_tree, accessibility_tree
- elements[] containing selector_candidates, tag, id, classes, role, accessible_name, text, attributes, bounding_box, visible, enabled, checked, selected, computed_styles, event_listeners_if_available
- forms, links, buttons, inputs, images, scripts, stylesheets, framework_roots

This file documents the bridge-facing API; the existing in-tree `browser` module remains the deterministic static/offline validation runtime.
