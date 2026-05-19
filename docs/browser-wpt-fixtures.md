# Browser WPT-Like Fixtures

The native browser parity suite now has a small executable fixture runner:

```bash
cargo test --test browser_wpt_like
```

The runner lives under `tests/browser_wpt_like/`. Each fixture records:

- the closest WPT area;
- the behavior shape being checked;
- the local HTML/script or page route setup;
- the expected observable result;
- unsupported behavior for the same fixture family.

## Current Fixture Families

| Family | Local fixture | Current unsupported notes |
| --- | --- | --- |
| DOM events | `tests/browser_wpt_like/dom_events.rs` | trusted event flags, complete UIEvent subclasses |
| Selectors API | `tests/browser_wpt_like/selectors.rs` | full selector grammar, pseudo-classes, invalid selector taxonomy |
| Fetch/CORS | `tests/browser_wpt_like/fetch_cors.rs` | full fetch error taxonomy, streaming bodies |
| Module scripts | `tests/browser_wpt_like/modules.rs` | complete ESM namespace semantics, import maps |
| CSS/layout | `tests/browser_wpt_like/css_layout.rs` | CSS grid, complete visual formatting model |
| Timers/microtasks | `tests/browser_wpt_like/timers_microtasks.rs` | wall-clock scheduling, task-source prioritization matrix |
| Web Storage | `tests/browser_wpt_like/storage.rs` | quota errors, cross-document storage events |
| HTML tree construction | `tests/browser_wpt_like/html_tree.rs` | HTML5 error recovery matrix, doctype and namespace handling |
| Forms/default actions | `tests/browser_wpt_like/forms.rs` | constraint validation, form-associated custom elements |
| Navigation/history | `tests/browser_wpt_like/navigation_history.rs` | full session history traversal algorithm |
| Context storage/cookies | `tests/browser_wpt_like/storage_context.rs` | quota, storage partitioning by top-level site |
| Keyboard/pointer interaction | `tests/browser_wpt_like/keyboard_pointer.rs` | IME composition, complete PointerEvent coordinate model |
| Focus navigation | `tests/browser_wpt_like/focus.rs` | shadow-root focus delegation, platform-specific tab stops |
| File upload/download | `tests/browser_wpt_like/file_transfer.rs` | real filesystem file picker, streamed download bodies |
| Realtime channels | `tests/browser_wpt_like/realtime.rs` | real socket transport, binary frames and EventSource reconnection timing |
| Permissions/media APIs | `tests/browser_wpt_like/permissions_media.rs` | real device capture, constraint solving and live MediaStream tracks |
| Dialogs/clipboard | `tests/browser_wpt_like/dialog_clipboard.rs` | modal event-loop blocking, system clipboard and user activation gating |

## Current Negative/Error Fixtures

| Family | Local fixture | Locked behavior |
| --- | --- | --- |
| Selectors API | `tests/browser_wpt_like/selectors_errors.rs` | invalid selectors currently return no matches instead of `SyntaxError` |
| Fetch/CORS | `tests/browser_wpt_like/fetch_cors_errors.rs` | missing `access-control-allow-origin` rejects fetch |
| Module scripts | `tests/browser_wpt_like/modules_errors.rs` | missing static import reports the resolved chunk URL |
| HTML tree construction | `tests/browser_wpt_like/html_tree_unsupported.rs` | table rows parse without implicit `tbody` insertion |

## Promotion Rule

Do not mark a browser surface as WPT-like unless it has a fixture here, a local
Rust assertion, unsupported-case notes, and coverage from
`cargo test --test browser_wpt_like`.
