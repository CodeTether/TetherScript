//! Registration of the identity and role built-ins.
//!
//! Separated from [`super::identity_install`] purely so each registration file stays
//! within the line budget; the split follows the concern boundary — request context
//! there, caller identity here.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::identity_claims;
use super::identity_gate;
use super::identity_headers::str_arg;
use super::identity_roles;
use super::identity_shape;
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `identity_from_claims`, `anonymous`, `has_role`, and `require_role`.
///
/// # Arguments
///
/// * `env` — Environment receiving the bindings.
///
/// # Returns
///
/// Nothing. `anonymous` and `has_role` are total, so they return a bare map and a
/// bare bool; the other two return `Result`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "identity_from_claims",
        pure_native("identity_from_claims", Some(1), |args| {
            Ok(wrap(identity_claims::from_claims(&args[0])))
        }),
        false,
    );
    bindings.define(
        "anonymous",
        pure_native("anonymous", Some(0), |_args| {
            Ok(identity_shape::anonymous())
        }),
        false,
    );
    bindings.define(
        "has_role",
        pure_native("has_role", Some(2), |args| {
            let role = str_arg(&args[1], "has_role: role")?;
            Ok(Value::Bool(identity_roles::holds(&args[0], &role)))
        }),
        false,
    );
    bindings.define(
        "require_role",
        pure_native("require_role", Some(2), |args| {
            let role = str_arg(&args[1], "require_role: role")?;
            Ok(wrap(identity_gate::require(&args[0], &role)))
        }),
        false,
    );
}
