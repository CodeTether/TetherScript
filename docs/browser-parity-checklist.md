# Browser Parity Checklist

This is the working checklist for making the native tetherscript browser a
preferred agent browser. Do not answer "what next" from memory; use this file,
`docs/browser-wpt-map.md`, and the named tests below.

## Decision Rules

- No Chromium, WebDriver, Playwright, or remote-control browser fallback.
- A capability is not ready until it has a local contract test.
- Browser-network behavior must be visible in route logs and HAR output.
- Production React/Vite flows are the smoke target, not handcrafted demos only.
- Keep changes zero-dependency unless a dependency is explicitly approved.

## Done Enough To Build On

| Area | Current anchor |
| --- | --- |
| React-style module mount | `tests/agent_browser_react_render.rs` |
| Controlled form interaction | `tests/agent_browser_react_interaction.rs` |
| Production debug report | `tests/agent_browser_production_debug.rs` |
| Route-backed module loader | `tests/agent_browser_modules.rs`, `tests/agent_browser_dynamic_import.rs` |
| Fetch/XHR auth cookies | `tests/agent_browser_auth_cookies.rs` |
| Fetch/XHR redirects | `tests/agent_browser_network_redirects.rs` |
| Fetch/XHR CORS credentials | `tests/agent_browser_cors_credentials.rs` |
| Routed page subresources | `tests/agent_browser_resource_network.rs` |
| Routed document navigation | `tests/agent_browser_navigation_network.rs` |
| Browser capability contract | `tests/browser_cap_contract.rs`, `tests/browser_cap_live.rs` |

## Blocking Checklist

### P0: One Network Pipeline For External Page Resources

Status: complete for external script entries, module script entries,
stylesheets, images, source maps, same-origin cookies, redirects, CORS-checked
allowed cross-origin resources, route logs, and HAR output. Top-level document
navigation remains separate and is tracked in P1.

Fetch/XHR now have cookies, redirects, CORS, credentials, and HAR. External
scripts, module script entries, CSS, images, and source maps use the same
route-visible network model before deterministic resource registration and
inlining.

Completed tests:

- module script loaded through a routed redirect chain records every HAR hop;
- module script request sends cookies only when browser rules allow it;
- stylesheet and image requests appear in HAR with route metadata;
- missing subresource error names the element and resolved URL.

### P1: Navigation Uses Browser Network Semantics

Status: complete for JavaScript `location` changes, anchor clicks, GET/POST
form submissions, redirects, response cookies, POST body preservation across
`307`/`308`, final-URL history commits, lifecycle events, route logs, and HAR
navigation entries.

Completed tests:

- `location.href` follows a routed redirect and commits the final URL;
- anchor click records unload/load lifecycle and HAR navigation entries;
- form POST preserves body, cookies, redirect behavior, and final URL;
- back/forward preserve session history after redirected navigations.

### P2: Module Loader Semantics

Status: complete for route-backed static imports, nested dependency-first
evaluation, modulepreload request deduplication, and browser-shaped rejection
for missing literal dynamic import chunks.

Completed tests:

- static imports fetch through the shared network pipeline;
- dynamic `import()` rejects with a browser-shaped error for missing chunks;
- modulepreload is requested, logged, and deduplicated with later imports;
- module evaluation order matches dependency order across nested imports.

### P3: Runtime Error And Async Diagnostics

Status: complete for browser-shaped unhandled promise rejection page errors,
production debug report projection, async generated script URL stack evidence,
CORS versus route-abort exception classification, and source-map lookup across
async and module stack frames.

Completed tests:

- unhandled promise rejection appears in page errors and debug report;
- async `await` stack includes the generated script URL;
- fetch CORS rejection is classified separately from route abort;
- source-map lookup covers async and module stack frames.

### P4: DOM And Event Parity For App Frameworks

React-style apps depend on DOM/event details beyond the current smoke surface.

Status: complete. Composed events dispatched inside an open shadow root now
bubble through the host chain and expose a composed path that includes the
shadow target, shadow root, host, and document. Element `children` and
`childNodes` collections now update after mutation for `length`, indexed access,
`item()`, and `forEach()`. Document-wide `getElementsBy*` queries and named
HTMLCollections such as `document.forms` now update after DOM and attribute
mutation, including dynamic named property lookup. Label clicks now activate
associated controls, native anchor clicks update `location.href`, and
`form.submit()` no longer dispatches a submit event while `requestSubmit()`
does. `requestSubmit(submitter)` now preserves the submitter's name/value pair
in collected form data. Capture, bubble, `stopPropagation`, and
`preventDefault` ordering matches browser behavior. Button and input reset
defaults restore form control values to their initial state, dispatching a
cancelable `reset` event first.

Completed tests:

- composed path across shadow DOM and nested targets.
- element `children` and `childNodes` update after mutation.
- document-wide live `getElementsBy*` and named HTMLCollections update after
  mutation.
- label activation, anchor `location.href` updates, and `form.submit()` versus
  `requestSubmit()` behavior.
- `requestSubmit(submitter)` includes submitter-specific form data.
- capture, bubble, `stopPropagation`, and `preventDefault` ordering.
- button reset restores text input, checkbox, and event cancelability.

### P5: CSS/Layout/Rendering Inspection

Agents need rendered-state evidence that matches production UI layout enough to
debug hidden, clipped, disabled, or overlapped controls.

Status: complete. Flex, absolute positioning, z-index, overflow clipping,
viewport-responsive width, display:none, visibility:hidden, and zero-size
elements are verified through agent-visible `production_debug_report()`
visual evidence. Text metrics are deterministic via 1px-per-character
measurement.

Completed tests:

- flex, position, z-index, and inline layout interact in one fixture.
- viewport resize changes computed layout evidence.
- text metrics have deterministic behavior.
- visual evidence marks zero-sized, display:none, visibility:hidden, and
  overflow-clipped controls.

### P6: WPT-Like Harness

The map in `docs/browser-wpt-map.md` needs executable fixtures so parity claims
are mechanically checked.

Status: complete for an initial executable fixture harness. The targeted suite
now lives in `tests/browser_wpt_like/` and runs with
`cargo test --test browser_wpt_like` in CI. It covers DOM events, Selectors API,
Fetch/CORS, module scripts, CSS/layout, timers/microtasks, Web Storage, and
HTML tree construction, including negative/error cases, with WPT-area metadata
and unsupported behavior notes in `docs/browser-wpt-fixtures.md`.

Completed tests:

- DOM event capture/target/bubble order and default prevention.
- Selectors API class and attribute matching through `querySelector*`.
- Fetch/CORS cross-origin POST preflight and response validation.
- Module-script static import graph loading and dependency-first evaluation.
- CSS/layout flex evidence through agent-visible visual diagnostics.
- Timers/microtasks deterministic drain ordering.
- Web Storage set/get/remove/clear length and key behavior.
- HTML tree construction for element order and entity text.
- Invalid selector behavior is locked as an unsupported no-match gap.
- Blocked CORS responses reject with missing-header diagnostics.
- Missing static module imports report the resolved chunk URL.
- HTML parser gaps include missing implicit `tbody` insertion.

## Immediate Next Item

Add WPT-like fixtures for form controls/default actions, navigation/history,
and cookie/storage context isolation.
