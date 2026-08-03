//! Built-in registration for the A/B test group.
//!
//! Split from `abtest.rs` so the entry point carries only documentation and module
//! declarations, matching how the `ratelimit` group separates installation.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::abtest_ops as ops;
use crate::system::result_value;
use crate::value::Env;

/// Define every A/B test built-in in `env`.
///
/// # Arguments
///
/// * `env` — Global environment being populated.
///
/// # Returns
///
/// Nothing. Three `Result`-returning built-ins and `ab_bucket` are defined.
/// `ab_bucket` returns a bare int rather than a `Result`, because a bucket cannot
/// fail once its two string arguments are accepted, and forcing a `?` on a value a
/// test uses in a tight loop would only add noise; a bad argument is still a real
/// error, raised the same way any other built-in raises one.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "ab_experiment",
        pure_native("ab_experiment", Some(1), |args| {
            Ok(result_value(ops::experiment(&args[0])))
        }),
        false,
    );
    bindings.define(
        "ab_assign",
        pure_native("ab_assign", Some(2), |args| {
            Ok(result_value(ops::assign(&args[0], &args[1])))
        }),
        false,
    );
    bindings.define(
        "ab_assign_from_request",
        pure_native("ab_assign_from_request", Some(2), |args| {
            Ok(result_value(ops::from_request(&args[0], &args[1])))
        }),
        false,
    );
    bindings.define(
        "ab_bucket",
        pure_native("ab_bucket", Some(2), |args| ops::bucket(&args[0], &args[1])),
        false,
    );
}
