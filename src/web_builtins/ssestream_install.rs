//! Registration of the streaming SSE built-ins.
//!
//! Split from the group root purely to respect the 50-line file limit: the root
//! keeps the module docs and the submodule declarations, and this file keeps the
//! flat list of `define` calls so adding a built-in touches one place.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::ssestream_args as args;
use crate::system::result_value;
use crate::value::Env;

/// Define the five streaming built-ins in `env`.
///
/// # Arguments
///
/// * `env` — Global environment.
///
/// # Returns
///
/// Nothing; the names are defined as immutable bindings.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut b = env.borrow_mut();
    b.define(
        "sse_stream_response",
        pure_native("sse_stream_response", Some(1), |a| {
            Ok(result_value(args::stream_response(a)))
        }),
        false,
    );
    b.define(
        "sse_stream_headers",
        pure_native("sse_stream_headers", Some(0), |_a| {
            Ok(args::stream_headers())
        }),
        false,
    );
    b.define(
        "sse_chunk",
        pure_native("sse_chunk", Some(1), |a| {
            Ok(result_value(args::stream_chunk(a)))
        }),
        false,
    );
    b.define(
        "sse_keepalive",
        pure_native("sse_keepalive", Some(0), |_a| Ok(args::keepalive())),
        false,
    );
    b.define(
        "sse_retry_frame",
        pure_native("sse_retry_frame", Some(1), |a| {
            Ok(result_value(args::retry_frame(a)))
        }),
        false,
    );
}
