# Upstream WPT fixtures

These files are copied without semantic modification from the canonical
[`web-platform-tests/wpt`](https://github.com/web-platform-tests/wpt) repository.

| Local fixture | Upstream path | Pinned commit | Blob |
| --- | --- | --- | --- |
| `fixtures/dom/nodes/DocumentFragment-querySelectorAll-after-modification.html` | `dom/nodes/DocumentFragment-querySelectorAll-after-modification.html` | `306b7ef2778d9673bf5db1acf7dd2cd1482abda7` | `8049363885788a1b13125c739788bebcde4d5a0e` |
| `fixtures/dom/nodes/Element-childElementCount-dynamic-add.html` | `dom/nodes/Element-childElementCount-dynamic-add.html` | `c4b340f055ec970af376bf81f644d12912bdf5bb` | `3e7490b21d6c290fdc77d430b529dcb9014e27d6` |

The adjacent Rust runner replaces only WPT infrastructure scripts with a small
local compatibility layer. Test source remains visible for provenance review.
