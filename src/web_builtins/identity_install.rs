//! Registration of the request-context built-ins.
//!
//! Split from `identity.rs` so the group root carries only documentation and module
//! declarations, matching how `ratelimit` and `header` separate installation. The
//! identity/role half lives in [`super::identity_install_auth`], because one file
//! registering seven built-ins would exceed the line budget.
//!
//! Every fallible built-in returns a `Result` through `result_value`, so a script
//! reaches the failure with `?` or `.err()` rather than through a panic. The bool
//! built-ins have no error channel by design and fail closed; see their modules.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::identity_context;
use super::identity_headers::str_arg;
use super::identity_install_auth;
use super::identity_session_ip;
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define the context built-ins, then delegate the identity half.
///
/// # Arguments
///
/// * `env` — Environment receiving the bindings.
///
/// # Returns
///
/// Nothing. Defines `request_context`, `request_id`, and `ip_changed` here, and the
/// remaining four through [`identity_install_auth::install`].
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    {
        let mut bindings = env.borrow_mut();
        bindings.define(
            "request_context",
            pure_native("request_context", Some(1), |args| {
                Ok(wrap(identity_context::extract(&args[0])))
            }),
            false,
        );
        bindings.define(
            "request_id",
            pure_native("request_id", Some(1), |args| {
                Ok(wrap(identity_context::id_of(&args[0])))
            }),
            false,
        );
        bindings.define(
            "ip_changed",
            pure_native("ip_changed", Some(2), |args| {
                let current = str_arg(&args[1], "ip_changed: current_ip")?;
                let moved = identity_session_ip::changed(&args[0], &current);
                Ok(Value::Bool(moved))
            }),
            false,
        );
    }
    identity_install_auth::install(env);
}
