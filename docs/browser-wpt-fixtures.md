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

## Promotion Rule

Do not mark a browser surface as WPT-like unless it has a fixture here, a local
Rust assertion, unsupported-case notes, and coverage from
`cargo test --test browser_wpt_like`.
