//! Environment registration for the password group.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::password_args::{needs_rehash_args, str_arg, verify_args};
use super::password_ops::hash;
use crate::system::result_value;
use crate::value::{Env, Value};

/// Register `password_hash`, `password_verify`, and `password_needs_rehash`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "password_hash",
        pure_native("password_hash", Some(1), |args| {
            Ok(result_value(
                str_arg(&args[0], "password_hash: password")
                    .map(|password| Value::Str(Rc::new(hash(&password)))),
            ))
        }),
        false,
    );
    bindings.define(
        "password_verify",
        pure_native("password_verify", Some(2), |args| {
            Ok(result_value(verify_args(args)))
        }),
        false,
    );
    bindings.define(
        "password_needs_rehash",
        pure_native("password_needs_rehash", Some(2), |args| {
            Ok(result_value(needs_rehash_args(args)))
        }),
        false,
    );
}
