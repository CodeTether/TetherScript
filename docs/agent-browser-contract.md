# Agent Browser Contract

The tetherscript browser track is the preferred browser surface for agents when
the job needs deterministic observation, explicit capabilities, replayable
actions, and testable output.

It has two stable entry points:

- `tetherscript render` and `tetherscript raster` for deterministic offline
  HTML/CSS rendering.
- `tetherscript::browser_agent::BrowserPage` for embedded observe, act, assert,
  screenshot, event-log, trace, and deterministic resource-validation workflows.

Browser control remains capability-gated through `BrowserAuthority` and
tetherscript browser action endpoints. Scripts must be granted origins and
scopes before navigation, interaction, network inspection, storage mutation,
visual lookup, or screenshot actions can cross that boundary.

Promotion rule: every agent-browser behavior that is recommended in docs needs
a local contract test. CLI behavior belongs in `tests/agent_browser_cli.rs`.
Embedded page behavior belongs in `tests/agent_browser_page.rs`. Action-envelope
behavior belongs in `tests/browser_cap_contract.rs`, and endpoint smoke tests
remain opt-in through `TETHERSCRIPT_BROWSERCTL_ENDPOINT`.
Current parity work is tracked in `docs/browser-parity-checklist.md`; use that
checklist before choosing the next browser task.

The browser track must not depend on external browser engines or automation
adapters. Production UI validation is done by closing gaps in tetherscript's
native DOM, CSS, JavaScript, networking, storage, accessibility, screenshot, and
trace implementation.

`BrowserPage::production_debug_report()` is the preferred one-shot native
debug artifact for production React-style pages. It now includes visual element
evidence that joins selector candidates, computed styles, visibility, and
layout bounds so an agent can distinguish "React mounted" from "React mounted
but the UI is hidden, zero-sized, or styled incorrectly."

Runtime diagnostics must preserve browser-shaped failure evidence. Unhandled
promise rejections are recorded as page errors, async and module stack frames
retain generated script URLs for source-map lookup, and CORS blocks are
classified separately from route aborts in production debug reports.

Agent actions must behave like user input for production form workflows:
controlled inputs read live DOM values from captured element handles, click
actions dispatch pointer/mouse/focus/click ordering, submit prevention blocks
native navigation, and Enter on an input submits the containing form through the
native runtime.

Auth-bearing network flows must preserve browser cookie semantics across the
native route layer. `fetch` and XHR responses that include `Set-Cookie` update
the page session jar, `HttpOnly` cookies stay hidden from `document.cookie`, and
later same-session requests receive a generated `Cookie` header.

Redirecting network flows must follow normal browser request semantics. Native
`fetch` and XHR follow `301`, `302`, `303`, `307`, and `308` responses, resolve
relative `Location` headers against the current hop, refresh cookies on each
hop, expose the final response URL, and preserve each hop in HAR output.

Cross-origin network flows must enforce browser CORS behavior after the host
capability policy permits the target origin. Native `fetch` and XHR add
`Origin`, run preflight `OPTIONS` requests for non-simple methods or headers,
validate `Access-Control-Allow-*` response headers, suppress cross-origin
cookies by default, and send credentialed cookies only for `credentials:
"include"` or `withCredentials = true`.

External page resources must use the native route-visible network path before
the deterministic registry/inlining path runs. Missing script, module entry,
stylesheet, image, and source-map resources can be fulfilled by routes, follow
redirects, apply same-origin cookies, enforce CORS for allowed cross-origin
resources, and appear in HAR output.

Top-level document navigation must use the native route-visible network path.
JavaScript `location` changes, anchor default actions, and GET/POST form
submits follow redirects, apply response cookies, preserve POST bodies across
`307`/`308`, commit the final URL into page history, and appear in HAR output.

Module loading must also stay route-visible. Static imports discovered from
module scripts are fetched through the native page-resource route path, passive
`modulepreload` fetches are deduplicated with later imports, nested static
dependencies evaluate before importers, and missing literal dynamic import
chunks reject with a browser-shaped `TypeError`.

## Readiness Suite

Run the deterministic and bridge-contract browser tests with:

```bash
cargo test --test agent_browser_cli --test agent_browser_page \
  --test browser_cap_contract --test browser_assertions \
  --test browser_cli_grant --test browser_trace_contract \
  --test agent_browser_resources --test agent_browser_modules \
  --test agent_browser_bundle_exports --test agent_browser_dynamic_import \
  --test agent_browser_native_contract \
  --test agent_browser_auth_cookies \
  --test agent_browser_cors_credentials \
  --test agent_browser_network_redirects \
  --test agent_browser_navigation_network \
  --test agent_browser_resource_network \
  --test agent_browser_production_debug \
  --test browser_js_promise_await --test browser_js_xhr_parity
```

Run the endpoint smoke test only when a tetherscript browser endpoint is
available:

```bash
TETHERSCRIPT_BROWSERCTL_ENDPOINT=http://127.0.0.1:41707/browser \
  cargo test --test browser_cap_live
```

The live host must accept action envelopes for the documented methods and
return either raw JSON, `{ "ok": true, "value": ... }`, or CodeTether-style
`{ "success": true, "output": ... }` tool results.
