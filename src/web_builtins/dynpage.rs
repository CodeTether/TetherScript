//! Dynamic page resolution and cache-key derivation.
//!
//! The reference application registers a dynamic-page middleware
//! (a `dynamic_page` middleware) that turns a request
//! path into a database-backed page — a CMS-style slug lookup — and then caches
//! the render. The port has no equivalent, so every such route would hand-roll
//! slug parsing and cache keys. That is dangerous work to duplicate: a cache key
//! that omits a varying input is exactly how one visitor is served another
//! visitor's page.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `slug_parse(path)` | `Result` of the normalised slug str |
//! | `slug_valid(slug)` | bool |
//! | `page_cache_key(parts)` | `Result` of the cache-key str |
//! | `vary_headers(parts)` | `Result` of the `Vary` header value str |
//! | `page_not_modified(cached_etag, request)` | `Result` of `nil` or a 304 map |
//! | `device_class(request)` | `mobile`, `tablet`, or `desktop` |
//! | `locale_of(request, supported)` | `Result` of the negotiated locale str |
//!
//! # Composition, not duplication
//!
//! `route.rs` already owns route pattern matching, path segmentation, and
//! percent-decoding, so nothing here re-implements them. A slug is *one* path
//! segment: for `/blog/{slug}` a caller uses `route_match` — or `path_segments` —
//! to obtain the already-decoded segment, and hands that to `slug_parse`. Because
//! decoding is `route.rs`'s job, a surviving `%` reaching `slug_parse` is
//! **rejected** rather than decoded: decoding a second time here would
//! re-introduce exactly the decode-before-split ordering bug that
//! `route_decode.rs` documents, letting `%2F` become a real separator.
//!
//! # Security
//!
//! * A slug is attacker-controlled and may reach a filesystem path or a template
//!   name, so `..`, `/`, `\`, NUL, and every percent-encoded form of them are
//!   rejected. See [`dynpage_reject`] for why rejecting beats sanitising.
//! * The cache key covers every input the render varies on, the authenticated
//!   flag included. See [`dynpage_key`].
//! * `vary_headers` lists exactly the headers the key consumed. See
//!   [`dynpage_vary`].
//! * `Accept-Language` parsing is bounded and never echoed back unvalidated. See
//!   [`dynpage_locale_parse`].
//!
//! # Examples
//!
//! ```tether
//! fn handle(req) {
//!     let parts = map()
//!     parts.slug = slug_parse(req.path)?
//!     parts.locale = locale_of(req, ["en", "es"])?
//!     parts.device = device_class(req)
//!     parts.authenticated = false
//!     let key = page_cache_key(parts)?
//!     let vary = vary_headers(parts)?
//!     let hit = cache_get(key)
//!     if hit != nil {
//!         let fresh = page_not_modified(hit.etag, req)?
//!         if fresh != nil { return fresh }
//!     }
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "dynpage_args.rs"]
mod dynpage_args;
#[path = "dynpage_charset.rs"]
mod dynpage_charset;
#[path = "dynpage_device.rs"]
mod dynpage_device;
#[path = "dynpage_install.rs"]
mod dynpage_install;
#[path = "dynpage_install_page.rs"]
mod dynpage_install_page;
#[path = "dynpage_key.rs"]
mod dynpage_key;
#[path = "dynpage_locale.rs"]
mod dynpage_locale;
#[path = "dynpage_locale_parse.rs"]
mod dynpage_locale_parse;
#[path = "dynpage_negotiate.rs"]
mod dynpage_negotiate;
#[path = "dynpage_notmod.rs"]
mod dynpage_notmod;
#[path = "dynpage_parts.rs"]
mod dynpage_parts;
#[path = "dynpage_reject.rs"]
mod dynpage_reject;
#[path = "dynpage_request.rs"]
mod dynpage_request;
#[path = "dynpage_slug.rs"]
mod dynpage_slug;
#[path = "dynpage_validator.rs"]
mod dynpage_validator;
#[path = "dynpage_vary.rs"]
mod dynpage_vary;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
///
/// # Returns
///
/// Nothing. Defines `slug_parse`, `slug_valid`, `device_class`, `locale_of`,
/// `page_cache_key`, `vary_headers`, and `page_not_modified` in `env`.
///
/// # Errors
///
/// None. Registration cannot fail; each built-in reports its own argument errors
/// as a `Result` at call time.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    dynpage_install::install(env);
    dynpage_install_page::install(env);
}
