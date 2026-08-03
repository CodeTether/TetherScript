//! ETag and cache-header built-ins.
//!
//! The port serves static assets and rendered pages but cannot answer a
//! conditional request, so every reload transfers the whole body. These built-ins
//! supply the two pieces that were missing: an entity tag derived from the body,
//! and a `Cache-Control` value with the directives the reference middleware in
//! a real application actually emits.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `etag_of(body)` | str, quoted strong validator |
//! | `etag_weak(body)` | str, `W/`-prefixed weak validator |
//! | `etag_matches(if_none_match, etag)` | bool |
//! | `cache_control(options)` | Result of str |
//! | `not_modified_response()` | map with status 304 |
//!
//! # Comparison is exact, never a substring test
//!
//! `etag_matches` compares each list entry after trimming and after stripping
//! `W/` and quotes, so `W/"x"` matches `"x"` per the RFC 9110 weak comparison
//! rule, while `"abc"` does **not** match `"abcdef"`. A prefix-only hit would
//! serve stale content, which is worse than not caching at all.
//!
//! # Examples
//!
//! ```tether
//! let body = "<h1>hello</h1>"
//! let tag = etag_of(body)
//!
//! if etag_matches(req.headers["if-none-match"], tag) {
//!     return not_modified_response()
//! }
//!
//! let opts = map()
//! opts.public = true
//! opts.max_age = 300
//! let resp = map()
//! resp.status = 200
//! resp.body = body
//! let headers = map()
//! headers["etag"] = tag
//! headers["cache-control"] = cache_control(opts)?
//! resp.headers = headers
//! ```
//!
//! # Layout
//!
//! * `etag_args` — argument coercion and the 304 response map
//! * `etag_tag` — SHA-256 validators and header comparison
//! * `etag_cache` — `Cache-Control` assembly
//! * `etag_options` — typed option accessors

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "etag_args.rs"]
mod etag_args;
#[path = "etag_cache.rs"]
mod etag_cache;
#[path = "etag_options.rs"]
mod etag_options;
#[path = "etag_tag.rs"]
mod etag_tag;

/// Register this group's built-ins.
///
/// Defines `etag_of`, `etag_weak`, `etag_matches`, `cache_control`, and
/// `not_modified_response` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "etag_of",
        pure_native("etag_of", Some(1), etag_args::of),
        false,
    );
    bindings.define(
        "etag_weak",
        pure_native("etag_weak", Some(1), etag_args::weak),
        false,
    );
    bindings.define(
        "etag_matches",
        pure_native("etag_matches", Some(2), etag_args::matches),
        false,
    );
    bindings.define(
        "cache_control",
        pure_native("cache_control", Some(1), |args| {
            Ok(result_value(etag_args::cache_control(args)))
        }),
        false,
    );
    bindings.define(
        "not_modified_response",
        pure_native("not_modified_response", Some(0), |_args| {
            Ok(etag_args::not_modified())
        }),
        false,
    );
}
