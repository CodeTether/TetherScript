//! Actix-compatible route pattern matching.
//!
//! This is the gate that blocks the the reference application port. The dispatcher in
//! `examples/the reference application/server/router.tether` compares `req.path` with `==`,
//! so it cannot express a parameterized route at all — and most real routes are
//! parameterized: `/customers/{id}`, `/api/short-urls/{code}/analytics`,
//! `/ab-tests/{id}/variants/{variant_id}`.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `route_match(pattern, path)` | map of captures, or `nil` when no match |
//! | `route_params(pattern)` | list of declared names, in order |
//! | `path_segments(path)` | list of decoded, non-empty segments |
//!
//! # Supported syntax
//!
//! * `{name}` captures exactly one segment and **never** spans `/`.
//! * `{name:.*}` as the final segment captures the remainder, separators included.
//! * Any other regex is rejected rather than silently mismatched.
//!
//! # No match versus error
//!
//! A router tries many patterns per request and expects most to fail, so a
//! non-match is `nil`, not an `Err`. `Err` means the *pattern* is malformed,
//! which is a program bug. This keeps `?` meaningful during dispatch.
//!
//! # Trailing slashes
//!
//! Empty segments are dropped, so `/a/b` and `/a/b/` are equivalent and
//! `/customers/{id}` matches `/customers/7/`. Actix distinguishes them by default
//! and offers `NormalizePath` to merge; folding them here avoids a 404 caused by a
//! stray slash in a hand-written dispatcher. See [`route_segments`] for the full
//! rationale.
//!
//! # Examples
//!
//! ```tether
//! let captures = route_match("/customers/{id}", "/customers/42")
//! if captures != nil { println(captures.id) }        // 42
//!
//! println(route_params("/ab-tests/{id}/variants/{variant_id}"))
//! println(path_segments("/customers/a%20b"))         // ["customers", "a b"]
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "route_args.rs"]
mod route_args;
#[path = "route_decode.rs"]
mod route_decode;
#[path = "route_match.rs"]
mod route_match;
#[path = "route_segments.rs"]
mod route_segments;
#[path = "route_tail.rs"]
mod route_tail;

/// Register this group's built-ins.
///
/// Defines `route_match`, `route_params`, and `path_segments` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "route_match",
        pure_native("route_match", Some(2), |args| {
            Ok(result_value(route_args::match_builtin(args)))
        }),
        false,
    );
    bindings.define(
        "route_params",
        pure_native("route_params", Some(1), |args| {
            Ok(result_value(route_args::params_builtin(args)))
        }),
        false,
    );
    bindings.define(
        "path_segments",
        pure_native("path_segments", Some(1), |args| {
            Ok(result_value(route_args::segments_builtin(args)))
        }),
        false,
    );
}
