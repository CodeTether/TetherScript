//! Registration of the cache-key, `Vary`, and conditional-response built-ins.
//!
//! All three consume a *derived* value rather than a raw request, which is why they
//! sit apart from the request-reading registrations in [`super::dynpage_install`].
//!
//! Each returns a `Result`, so a malformed parts map surfaces as `Err` and a
//! handler can propagate it with `?`. That matters most for `page_cache_key`: a
//! silently-accepted bad input there would produce a key that collides with a real
//! one, which is precisely the failure mode this group exists to prevent.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::dynpage_args::str_arg;
use super::dynpage_key;
use super::dynpage_notmod;
use super::dynpage_parts;
use super::dynpage_vary;
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `page_cache_key`, `vary_headers`, and `page_not_modified` in `env`.
///
/// # Arguments
///
/// * `env` — Environment receiving the bindings.
///
/// # Returns
///
/// Nothing.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "page_cache_key",
        pure_native("page_cache_key", Some(1), key),
        false,
    );
    bindings.define(
        "vary_headers",
        pure_native("vary_headers", Some(1), vary),
        false,
    );
    bindings.define(
        "page_not_modified",
        pure_native("page_not_modified", Some(2), fresh),
        false,
    );
}

/// `page_cache_key(parts)` -> `Result` of the derived key str.
fn key(args: &[Value]) -> Result<Value, String> {
    Ok(wrap(dynpage_parts::read(&args[0], "page_cache_key").map(
        |parts| Value::Str(Rc::new(dynpage_key::build(&parts))),
    )))
}

/// `vary_headers(parts)` -> `Result` of the `Vary` header value str.
fn vary(args: &[Value]) -> Result<Value, String> {
    Ok(wrap(dynpage_parts::read(&args[0], "vary_headers").map(
        |parts| Value::Str(Rc::new(dynpage_vary::build(&parts))),
    )))
}

/// `page_not_modified(cached_etag, request)` -> `Result` of nil or a 304 map.
fn fresh(args: &[Value]) -> Result<Value, String> {
    Ok(wrap(
        str_arg(&args[0], "page_not_modified: cached_etag")
            .and_then(|etag| dynpage_notmod::decide(&etag, &args[1], "page_not_modified")),
    ))
}
