# Browser Capability API

TetherScript has a browser authority (`browser_cap::BrowserAuthority`) designed
as the seam between scripts and native tetherscript browser infrastructure. It
is not an adapter to another browser engine. The authority posts compact action
envelopes to a tetherscript browser host and records an in-process trace for
audit and replay.

## Granting

Start the in-tree native browser action host:

```bash
cargo run --bin tetherscript-browser-host -- 127.0.0.1:41707
```

Then grant its endpoint to a script:

```bash
tetherscript run --interp \
  --grant-browser http://127.0.0.1:41707/browser \
  --browser-origin http://localhost:5173 \
  --browser-scope all \
  examples/browser_agentic_debug.tether
```

The host owns one persistent `BrowserPage` and exits after `browser.stop()`.
It executes actions through tetherscript's own DOM, JavaScript, layout, and rendering stack.
Snapshot `visible_text` and text waits exclude non-rendered DOM subtrees.

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

## Native Host Contract

For a method like `browser.goto(url)`, TetherScript sends:

```http
POST /browser
Content-Type: application/json

{"action":"goto","url":"http://localhost:5173"}
```

The host may return either a raw JSON value or:

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

Live native-host smoke test:

```bash
TETHERSCRIPT_BROWSERCTL_ENDPOINT=http://127.0.0.1:41707/browser \
  cargo test --test browser_cap_live
```

The test starts a local fixture page, then drives the configured tetherscript
browser endpoint through start, goto, wait, eval, snapshot, screenshot, and
stop.

## MVP methods

Navigation/interaction:
`goto`, `reload`, `back`, `forward`, `click`, `click_text`, `type`, `press`,
`hover`, `focus`, `blur`, `scroll`, `wait_for_selector`, `wait_for_text`,
`wait_for_url`, `wait_for_network_idle`.

The wait helpers are tetherscript convenience methods over the host
`wait` action:

- `wait_for_selector(selector, timeout_ms?)` sends `{"action":"wait","selector":...,"state":"visible","timeout_ms":...}`.
- `wait_for_text(text, timeout_ms?)` sends `{"action":"wait","text":...,"timeout_ms":...}`.
- `wait_for_url(url_substring, timeout_ms?)` sends `{"action":"wait","url_contains":...,"timeout_ms":...}`.
- `wait_for_network_idle()` sends `{"action":"wait","network_idle":true}`.

The native host polls until the requested wall-clock timeout while settling the
deterministic page runtime between attempts. Raw selector waits support
`attached`, `detached`, `visible`, and `hidden` states.

`wait_for_network_idle` drains pending deterministic page scripts and requests
until the native page reaches its `NetworkIdle` lifecycle milestone.
`compare_screenshots` and `visual_diff` compare deterministic native screenshot
files and report match state, encoded byte counts, and changed encoded bytes.

`scroll` accepts `scroll(selector)`, `scroll(x, y)`, or
`scroll(selector, x, y)` and sends a `scroll` action envelope with the matching
fields. The native host brings a selector into view, treats bare coordinates as
absolute window offsets, and treats coordinates paired with a selector as an
offset from the resulting in-view position.

Snapshots:
`screenshot`, `screenshot_element`, `dom_snapshot`, `page_snapshot`.

Diagnostics:
`console_logs`, `console_errors`, `unhandled_rejections`, `runtime_exceptions`, `source_mapped_stack_traces`.
The native host returns captured console levels/messages, classified runtime
exceptions, and generated source locations with source-map frame counts.

Native page diagnostics:
`BrowserPage::production_debug_report()` returns a single structured report for
agent-side bundled UI debugging: console errors, page errors, failed requests,
HAR-style network entries, source-map registration status, source-mapped
page-error locations and generated stack frames, framework markers, React roots,
hydration warnings, classified runtime exceptions, and native visual element
evidence with selector candidates, computed styles, visibility, and layout
bounds. Unhandled promise rejections are promoted into page errors, async and
module frames retain generated script URLs for source-map lookup, and CORS
blocks are classified separately from route aborts. This is the native
full-parity track's production-debug surface; it does not depend on an external
browser engine or remote-control driver.

Native page actions are expected to exercise production form code paths:
`fill` updates live form values and dispatches input/change, `click` uses a
user-like pointer/mouse/focus/click sequence, prevented submits do not navigate,
and Enter on an input submits its containing form. The native host retains the
focused locator across action envelopes so `focus`, `click`, or `fill` can be
followed by `press`; `blur` and navigation clear that host focus. `type` appends
printable characters one at a time with cancelable keydown, input, and keyup
events, while `fill` replaces the existing value.
`keyboard_type` applies the same incremental event semantics to the element
retained by the latest focusable action.
`fill_native(selector, value)` replaces a form value through the native page
action path and dispatches the same input/change events as `fill`.
`upload` reads the supplied host paths, exposes deterministic file metadata to
an `input[type=file]`, and dispatches input and change events.
`toggle` inverts a checkbox or radio through its click, input, and change path.
`mouse_click` hit-tests viewport coordinates and dispatches a trusted pointer and
mouse sequence at the requested point.
Tabs retain independent page, history, scroll, and runtime state; selecting or
closing a tab swaps the active host page without reloading it.

Network:
`network_log`, `network_har`, `failed_requests`, `request`, `response`, `replay_request`, `wait_for_request`, `wait_for_response`.
The native host returns captured request method, URL, status, and route result;
`failed_requests` restricts the result to missing or error statuses.
`wait_for_request` and `wait_for_response` poll captured events while settling
the page runtime until a URL match appears or the requested timeout expires.
`fetch`, `axios`, and `xhr` execute through that same page network runtime and
return a normalized status, URL, method, body, success flag, and transport map.
`replay_request` reissues the latest URL match and can replace its request body.
Native fetch/XHR networking follows redirects, emits CORS preflights, validates
`Access-Control-Allow-*` headers, and models credential modes for cross-origin
cookies. External page resources use the same route-visible network model for
script, stylesheet, image, and source-map loads before deterministic inlining.
Top-level `location`, anchor, and form navigations also use the native route
model, including redirects, cookies, POST bodies, final URL commits, history,
and HAR entries.

Storage:
`cookies`, `local_storage`, `session_storage`, `indexed_db_summary`, `set_cookie`, `set_local_storage`, `clear_storage`.
`indexed_db_summary()` returns origin-scoped records with database, object-store,
key, and value fields from the native browser context.
`clear_storage()` clears cookies, localStorage, sessionStorage, IndexedDB, and
service-worker/cache state across every tab in the native browser context.

React/framework hooks:
Use string method syntax for dotted method names, e.g. `browser."react.detect"()?`, `browser."react.component_for_selector"("#root")?`, plus `frameworks()` for Next/Vite/Redux/Zustand/React Query detection returned by the host.
Selector-based component queries return stable DOM-backed tag, component-name,
text, attribute, and optional `data-react-*` diagnostic metadata.

Trace/export:
`trace`, `export_trace_json`, `export_har`, `agent_summary`, `minimal_reproduction_script`.

Raw action:
`raw(action, params)` sends an explicit host action envelope after the same
scope and origin checks as high-level methods. This is an FFI-style escape hatch;
the stable API remains the language-level browser methods above. Raw action
names are restricted to the native host's advertised action enum.

## Agent assertions

Runtime assertion helpers return `Result` values suitable for `?` propagation:

- `assert_selector(browser, selector)`
- `assert_text(browser, text)`
- `assert_no_console_errors(browser)`
- `assert_no_failed_requests(browser)`
- `assert_visible(browser, selector)`
- `assert_enabled(browser, selector)`
- `assert_route(browser, path_or_url_substring)`
- `assert_react_component(browser, name)`

`assert_screenshot_matches(browser, path)` compares the current native viewport
against a PNG baseline and returns an error naming the differing byte count.

## Embedded resource validation

`BrowserPage::validate_external_resources()` checks that every external script,
stylesheet, image, `modulepreload`, and preload reference in the page has a
registered deterministic resource. This lets agents fail fast on missing
production bundles before executing app code or taking screenshots.

When `BrowserPage::run_scripts()` is called, registered `<script src="...">`
resources, including `type="module"` entries, are inlined at their original
script element and executed by the in-tree JavaScript runtime in document order.
Static module imports are fetched through the route-visible page-resource path
when missing from the registry and executed before the importing module.
Passive `modulepreload` fetches are deduplicated with later imports. Relative
imports can match registered page paths or fully resolved URLs. Default imports,
named import aliases, `export default`,
and bundle-style `export { local as Name }` lists are rewritten into runtime
bindings for the existing JavaScript engine. Literal dynamic imports such as
`import("./chunk.js")` are resolved through the same registry and rewritten to a
fulfilled namespace promise; missing dynamic chunks return a rejected promise
with a browser-shaped `TypeError`. Common arrow functions in module resources
are rewritten to function expressions before evaluation. Passive preload links
are validated but do not execute by themselves.

## Page snapshot schema target

The native host should make `page_snapshot()` return a compact map with:

- url, title, viewport, scroll_position, focused_element, selected_element, visible_text
- dom_tree, accessibility_tree
- elements[] containing selector_candidates, tag, id, classes, role, accessible_name, text, attributes, bounding_box, visible, enabled, checked, selected, computed_styles, event_listeners_if_available
- forms, links, buttons, inputs, images, scripts, stylesheets, framework_roots

This file documents the host-facing API; the in-tree browser modules remain the
runtime that production browser validation must exercise and improve.
