//! Built-in registration for the rate-limit group.
//!
//! Split from `ratelimit.rs` so the entry point carries only documentation and
//! module declarations, matching how the `form` group separates installation.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::ratelimit_arg as arg;
use super::ratelimit_fields as fields;
use super::ratelimit_response as response;
use super::ratelimit_take as ops;
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define every rate-limit built-in in `env`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "bucket_new",
        pure_native("bucket_new", Some(2), |args| {
            let capacity = fields::num_arg(&args[0], "bucket_new: capacity")?;
            let refill = fields::num_arg(&args[1], "bucket_new: refill_per_sec")?;
            Ok(result_value(ops::new(capacity, refill)))
        }),
        false,
    );
    bindings.define(
        "bucket_take",
        pure_native("bucket_take", Some(2), |args| {
            let cost = fields::num_arg(&args[1], "bucket_take: cost")?;
            Ok(result_value(arg::take(&args[0], cost)))
        }),
        false,
    );
    bindings.define(
        "retry_after_header",
        pure_native("retry_after_header", Some(1), |args| {
            let ms = fields::num_arg(&args[0], "retry_after_header: retry_after_ms")?;
            Ok(Value::Int(response::header_seconds(ms as i64)))
        }),
        false,
    );
    bindings.define(
        "too_many_requests_response",
        pure_native("too_many_requests_response", Some(1), |args| {
            let label = "too_many_requests_response: retry_after_ms";
            let ms = fields::num_arg(&args[0], label)?;
            Ok(response::too_many_requests(ms as i64))
        }),
        false,
    );
}
