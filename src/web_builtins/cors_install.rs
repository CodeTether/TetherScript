//! Registration of the CORS built-ins.
//!
//! Split from `cors.rs` so the group root stays within the line budget and holds
//! only module docs and the `install` entry point.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::{cors_config, cors_preflight, cors_response};
// `result_value` is private to `crate::system`, so it is imported directly rather
// than re-exported through the group root, matching `header_install`.
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `cors_policy`, `cors_preflight`, `cors_headers`, and `is_preflight`.
///
/// # Arguments
///
/// * `env` — The global environment being populated.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "cors_policy",
        pure_native("cors_policy", Some(1), |args| {
            Ok(wrap(cors_config::build(&args[0])))
        }),
        false,
    );
    bindings.define(
        "cors_preflight",
        pure_native("cors_preflight", Some(2), |args| {
            Ok(wrap(cors_preflight::decide(&args[0], &args[1])))
        }),
        false,
    );
    bindings.define(
        "cors_headers",
        pure_native("cors_headers", Some(2), |args| {
            Ok(wrap(cors_response::build(&args[0], &args[1])))
        }),
        false,
    );
    bindings.define(
        "is_preflight",
        // Returns a bare bool, not a Result: "is this a preflight" has no failure
        // mode a script can act on, and `if is_preflight(req)` should read plainly.
        pure_native("is_preflight", Some(1), |args| {
            Ok(Value::Bool(cors_preflight::detect(&args[0])?))
        }),
        false,
    );
}
