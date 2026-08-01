//! Registration of the structured logging built-ins.
//!
//! Split from `log.rs` so the entry point stays a declaration list. Argument
//! coercion lives in `log_args`, leaving this file as the binding table.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::log_args;
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define every logging built-in in `env`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "log_json",
        pure_native("log_json", Some(3), |args| {
            Ok(result_value(log_args::log_json(args)))
        }),
        false,
    );
    bindings.define("log_debug", wrapper("log_debug", "debug"), false);
    bindings.define("log_info", wrapper("log_info", "info"), false);
    bindings.define("log_warn", wrapper("log_warn", "warn"), false);
    bindings.define("log_error", wrapper("log_error", "error"), false);
    bindings.define(
        "log_level_enabled",
        pure_native("log_level_enabled", Some(1), |args| {
            Ok(result_value(log_args::level_enabled(&args[0])))
        }),
        false,
    );
}

/// Build a fixed-level convenience wrapper such as `log_info`.
fn wrapper(name: &'static str, level: &'static str) -> Value {
    pure_native(name, Some(1), move |args| {
        Ok(result_value(log_args::log_at(level, name, args)))
    })
}
