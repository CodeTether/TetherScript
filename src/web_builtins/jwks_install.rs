//! Environment registration for the JWKS built-ins.
//!
//! Separated from `jwks.rs` so the owning module only declares its submodules and
//! its documentation, keeping every file inside the 50-line limit.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::jwks_args;
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define `jwks_parse`, `jwks_find`, `jwt_header`, and `jwt_rs256_parts`.
///
/// # Arguments
///
/// * `env` — Environment to populate.
///
/// # Returns
///
/// Nothing; the four names are bound as immutable built-ins.
///
/// # Errors
///
/// Cannot fail; each built-in reports its own failures as a script `Result`.
///
/// # Examples
///
/// ```tether
/// println(str(jwks_find([], "any").is_err()))   // true
/// ```
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("jwks_parse", parse(), false);
    bindings.define("jwks_find", find(), false);
    bindings.define("jwt_header", header(), false);
    bindings.define("jwt_rs256_parts", rs256_parts(), false);
}

fn parse() -> Value {
    pure_native("jwks_parse", Some(1), |args| {
        Ok(result_value(jwks_args::parse(args)))
    })
}

fn find() -> Value {
    pure_native("jwks_find", Some(2), |args| {
        Ok(result_value(jwks_args::find(args)))
    })
}

/// Bound as `jwt_header` rather than `jwt_header_unverified` only because the
/// task fixes the name; the doc comment on `super::jwks_parts::header` carries
/// the warning, and every error and test message repeats it.
fn header() -> Value {
    pure_native("jwt_header", Some(1), |args| {
        Ok(result_value(jwks_args::header(args)))
    })
}

fn rs256_parts() -> Value {
    pure_native("jwt_rs256_parts", Some(1), |args| {
        Ok(result_value(jwks_args::rs256_parts(args)))
    })
}
