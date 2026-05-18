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
| Static and dynamic module resources | `tests/agent_browser_modules.rs`, `tests/agent_browser_dynamic_import.rs` |
| Fetch/XHR auth cookies | `tests/agent_browser_auth_cookies.rs` |
| Fetch/XHR redirects | `tests/agent_browser_network_redirects.rs` |
| Fetch/XHR CORS credentials | `tests/agent_browser_cors_credentials.rs` |
| Routed page subresources | `tests/agent_browser_resource_network.rs` |
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

Top-level navigation and default actions need the same redirect, cookie, CORS,
history, and lifecycle behavior as fetch/XHR where the platform requires it.

Required tests:

- `location.href` follows a routed redirect and commits the final URL;
- anchor click records unload/load lifecycle and HAR navigation entries;
- form POST preserves body, cookies, redirect behavior, and final URL;
- back/forward preserve session history after redirected navigations.

### P2: Module Loader Semantics

The current module support is registry-backed and enough for React smoke tests,
but not a full browser module loader.

Required tests:

- static imports fetch through the shared network pipeline;
- dynamic `import()` rejects with a browser-shaped error for missing chunks;
- modulepreload is requested, logged, and deduplicated with later imports;
- module evaluation order matches dependency order across nested imports.

### P3: Runtime Error And Async Diagnostics

Production debugging needs browser-shaped rejection/error reporting and async
stack evidence, not just successful render output.

Required tests:

- unhandled promise rejection appears in page errors and debug report;
- async `await` stack includes the generated script URL;
- fetch CORS rejection is classified separately from route abort;
- source-map lookup covers async and module stack frames.

### P4: DOM And Event Parity For App Frameworks

React-style apps depend on DOM/event details beyond the current smoke surface.

Required tests:

- capture, bubble, `stopPropagation`, and `preventDefault` ordering;
- composed path across shadow DOM and nested targets;
- live collections update after mutation;
- default actions for labels, buttons, forms, and anchors stay coherent.

### P5: CSS/Layout/Rendering Inspection

Agents need rendered-state evidence that matches production UI layout enough to
debug hidden, clipped, disabled, or overlapped controls.

Required tests:

- flex, position, overflow, z-index, and inline layout interact in one fixture;
- viewport resize changes computed layout and screenshot evidence;
- fonts and text metrics have deterministic fallback behavior;
- visual evidence marks zero-sized, covered, and offscreen controls.

### P6: WPT-Like Harness

The map in `docs/browser-wpt-map.md` needs executable fixtures so parity claims
are mechanically checked.

Required tests:

- add `tests/browser_wpt_like/` fixture runner;
- add first fixture set for DOM events, selectors, fetch/CORS, and modules;
- document unsupported behavior per fixture family;
- run the targeted fixture subset in CI.

## Immediate Next Item

Implement P1: route top-level navigation and default navigation actions through
native browser network semantics, then add the four P1 tests above.
