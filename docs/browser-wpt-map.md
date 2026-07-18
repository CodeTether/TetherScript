# Browser WPT/Compliance Map

This inventory maps the native tetherscript browser parity track to Web
Platform Tests (WPT)-style coverage. Entries name the implemented surface, the
closest WPT area or test shape, local coverage, and known gaps that block
claiming full browser parity.

## Scope

The browser track is a native full-parity target for agents. It starts from a
deterministic in-tree implementation: HTML is parsed into a DOM, CSS is parsed
and matched, layout/rendering are computed natively, and browser APIs are
exposed to the in-tree JavaScript interpreter. Full parity means closing this
map against spec/WPT behavior without delegating execution to an external
browser engine or remote-control driver.

This document is not a pass/fail WPT report. It is a planning map for turning
local unit tests and fixtures into WPT-like compliance cases.

## Inventory

| Area | Implemented surface | WPT-like reference area | Local coverage anchor | Status | Main gaps |
| --- | --- | --- | --- | --- | --- |
| HTML tree construction | Elements, text nodes, attributes, void-ish `<br>`/`img`, basic entity decoding | `html/syntax/parsing`, `html/dom` | `src/browser.rs::parses_html_to_dom_value`, `decodes_nested_entities` | Partial | No HTML5 tokenizer/tree-construction algorithm, namespaces, comments/doctype, error recovery matrix, script/parser interaction |
| DOM querying | `querySelector`, `querySelectorAll`, `getElementById`, compound selectors, all four combinators, attribute matching, `:not`, and structural pseudo-classes | `dom/nodes`, `selectors-api` | `src/browser.rs::pseudo_nth_child_selects_by_position`, `tests/browser_wpt_like/selectors.rs` | WPT-like deterministic subset | No selector lists, namespaces, CSS escaping, dynamic-state/relational pseudos, pseudo-elements, or invalid-selector `SyntaxError` taxonomy |
| Selectors/CSS cascade | Type/id/class/attribute selectors, descendant compounds, specificity, inline `style` precedence | `css/selectors`, `css/css-cascade` | `src/browser.rs::css_supports_compound_descendant_and_inline_cascade` | Partial | No origins/layers/importance/media queries/inheritance matrix/shorthand expansion/unknown-value handling |
| CSSOM/layout/rendering | Basic dimensions, background/text/image display commands, deterministic text rendering, flex/position/overflow/z-index diagnostics, grid computed-style evidence, table span sizing evidence, transform/opacity stacking style evidence | `css/cssom`, `css/css2`, `css/css-grid`, `html/rendering` | `src/browser.rs::lays_out_and_renders_text_display_list`, `tests/browser_wpt_like/css_layout.rs` | WPT-like deterministic subset | Not a full CSS visual formatting model; grid/table assertions are diagnostic evidence rather than full placement algorithms; fonts and complete painting order remain partial |
| Canvas/WebGL | Native RGBA canvas pixels, ImageData reads/writes, WebGL color clears/readback, viewport/scissor state, channel write masks, and sticky GL errors | `html/canvas`, `webgl` | `src/browser_js_canvas/webgl_tests_state.rs`, `tests/browser_wpt_like/canvas_webgl.rs` | WPT-like software-rendered subset | Shaders, buffers, textures, draw calls, and GPU acceleration remain unsupported |
| Scrolling | Viewport wheel offsets; element `scroll`, `scrollTo`, `scrollBy`, `scrollLeft`, `scrollTop`, and `scrollIntoView`; clamped overflow geometry; synchronized `visualViewport` metrics plus trusted `resize`, `scroll`, and `scrollend` events; scrolled hit testing | `css/cssom-view`, `uievents/wheel` | `src/browser_js_dom/scroll_metrics_tests.rs`, `src/browser_js_viewport/tests_visual_viewport_events.rs`, `tests/browser_wpt_like/wheel_scroll.rs` | WPT-like deterministic subset | Smooth scrolling animation, pinch zoom, and scrollbar hit testing remain unsupported |
| DOM text/html attributes | `textContent`, `innerText`, `innerHTML`, `children`, attribute methods, and queued `MutationObserver` child/attribute/character-data records | `dom/nodes`, `html/dom/elements`, `dom/MutationObserver` | `src/browser_js_dom/tests_replace_children.rs`, `tests/browser_mutation_observer_parity.rs` | WPT-like deterministic subset | Attribute reflection and serialization remain partial; comment, CDATA, and processing-instruction nodes are absent |
| DOM mutation | `createElement`, `createDocumentFragment`, `appendChild`, prepend/remove helpers, `insertBefore`, `replaceChild`, `cloneNode`, `importNode`, and `adoptNode` | `dom/nodes`, `dom/nodes/ParentNode-*` | `src/browser_js_dom/node_relations/tests_node_methods.rs`, `tests/browser_dom_mutation_parity.rs` | WPT-like deterministic subset | Index-path handles can shift after mutation; import/adopt do not model cross-document realms |
| Events/Input | `addEventListener`, `removeEventListener`, `onclick`, `dispatchEvent`, `click`, event `type`/`target`, listener `this`, composed paths, browser-generated click trust flags, composition event data, touchstart/touchmove/touchend metadata, PointerEvent pointerType differentiation | `dom/events`, `uievents`, `pointerevents`, `touch-events` smoke cases | `src/browser_js.rs::event_listeners_property_handlers_this_and_event_target_work`, `tests/browser_wpt_like/dom_events.rs`, `tests/browser_wpt_like/keyboard_pointer.rs` | WPT-like deterministic subset | Unforgeable trusted-event boundaries and complete hardware input negotiation remain partial |
| Window/global aliases | `window`, `self`, `document`, `navigator`, `location`; screen orientation type/angle, locking, unlocking, trusted change events, and legacy `window.orientation`/`orientationchange` aliases | `html/webappapis`, `html/browsers/the-window-object`, `screen-orientation` | `src/browser_js.rs::location_and_navigator_globals_are_available`, `src/browser_js_viewport/tests_orientation_lock.rs`, `tests/browser_wpt_like/viewport_media.rs` | WPT-like deterministic subset | No physical display or device-sensor integration |
| Timers and microtasks | Deterministic `setTimeout`, `clearTimeout`, `setInterval`, `queueMicrotask`, `requestAnimationFrame`, Promise reactions, and callback args drained after script execution | `html/webappapis/timers`, `html/webappapis/microtask-queuing` | `src/browser_js.rs::microtasks_animation_frames_and_timers_have_deterministic_order`, `tests/browser_js_promise_await.rs` | WPT-like deterministic subset | No wall-clock scheduling, clamping/nesting behavior, full task-source model, or worker timers |
| Web Storage/resource guards | In-memory `localStorage`/`sessionStorage` with `getItem`, `setItem`, `removeItem`, `clear`, `key`, `length`, `navigator.storage.estimate()`, quota rejection, and deterministic memory-pressure cache trimming | `webstorage`, storage quota/resource pressure smoke | `src/browser_js.rs::local_storage_implements_minimal_storage_api`, `tests/browser_wpt_like/resource_guard.rs` | WPT-like deterministic subset | Property-indexed access and real OS memory-pressure signals remain unsupported |
| JavaScript integration | Inline `<script>` execution, expression return value, console log capture, functions/classes, loops, modern expression syntax, `typeof`, `this` in supported callbacks, deterministic module resource expansion for default/named imports and dynamic imports, Promise adoption, `await`, `fetch`, and `XMLHttpRequest` response lifecycle fields | `html/semantics/scripting-1`, `console`, `ecmascript` host smoke, `xhr`, `fetch` | `src/browser_js.rs` unit tests, `tests/agent_browser_react_render.rs`, `tests/browser_js_promise_await.rs`, `tests/browser_js_xhr_parity.rs` | Project-specific | No full ESM loader, complete Test262 semantics, complete XHR/fetch error taxonomy, exceptions parity, async stack traces, or external WPT harness |
| Runtime builtins | `browser_parse_html`, `browser_parse_css`, `browser_styles`, `browser_query_selector`, `browser_text_content`, `browser_snapshot`, `browser_display_list`, `browser_render`, `browser_layout`, `browser_run_scripts`, `browser_eval_js`, compatibility report | Project API contract; WPT harness adapter candidates | `src/browser.rs::browser_builtins_return_values`, `browser_variadics_reject_extra_args`; `src/browser_js.rs::compatibility_report_lists_storage_apis` | Local API covered | Need stable JSON fixture format and harness glue before importing external WPT data |
| Production diagnostics | Console/page errors, HAR-style network entries, source-mapped error locations and generated stack frames, failed requests, source-map references, classified runtime exceptions, React roots and hydration warnings | `console`, `fetch`, source maps, framework integration smoke | `tests/agent_browser_production_debug.rs` | Agent-debug subset | Needs async stack frames and framework component stack reconstruction |

## Executable WPT-like fixture layout

The initial executable fixture suite lives under `tests/browser_wpt_like/` and
runs with:

```bash
cargo test --test browser_wpt_like
```

The first fixture families cover DOM events, Selectors API, Fetch/CORS, module
scripts, CSS/layout, timers/microtasks, Web Storage, and HTML tree
construction. Form defaults, same-document history, context storage/cookies,
keyboard/pointer interaction (including composition/touch/pointerType), actionability stability, focus navigation, and file upload/download now
have fixture coverage too. Realtime channels, permissions/media APIs, and
dialog/clipboard behavior are now locked in the same runner. Frames/window
messaging, security policy, canvas/WebGL, and accessibility snapshots now have
fixture coverage too. Service workers/cache storage, IndexedDB, selection and
legacy editing commands,
screenshots/visual diff, viewport matchMedia listener dispatch, quota/resource guards, drag DataTransfer payloads, and page trace/persistence behavior are now covered in
the same fixture runner. Negative/error fixtures lock invalid selector, blocked
CORS, missing module chunk, unsupported HTML parser behavior, failed realtime
connections, denied media permissions, blocked frame messages, missing persisted
records, and invalid visual/selection locators. Unsupported behavior for each
family is documented in `docs/browser-wpt-fixtures.md`.

Future fixture families can add cases without importing WPT itself:

```text
tests/browser_wpt_like/
  dom-query-basic.json          # html, script/query, expected text/counts
  events-click-basic.json       # html, script, expected value/console/dom text
  storage-basic.json            # script, expected value
  timers-deterministic.json     # script, expected console/order
  css-cascade-basic.json        # html, css, selector, expected computed fields
```

Each fixture should use a small normalized schema:

```json
{
  "area": "dom/events",
  "wpt_shape": "dispatchEvent invokes listener with target and this",
  "html": "<button id='go'>old</button>",
  "script": "let b=document.getElementById('go'); let seen=''; b.addEventListener('click', function(e){ seen=e.type + ':' + e.target.id + ':' + this.id; }); b.click(); seen;",
  "expect": { "value": "click:go:go" }
}
```

## Promotion criteria

Before claiming WPT compatibility for any row, add or link at least:

1. a WPT-like fixture with spec/WPT-area metadata;
2. a local Rust harness assertion for the fixture result;
3. documented unsupported cases for the same feature family; and
4. a stable command that runs the targeted subset in CI.
