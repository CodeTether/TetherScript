//! UUID built-ins: `uuid_v4`, `uuid_parse`, and `uuid_is_valid`.
//!
//! A real web application needs UUID primary keys and request/correlation IDs.
//! Entropy is drawn the way [`crate::postgres`] does for SCRAM nonces — a fixed
//! read from `/dev/urandom`, falling back to time and PID — so the core build
//! stays dependency-free.
//!
//! # Examples
//!
//! ```tether
//! let id = uuid_v4()
//! if uuid_is_valid(id) { println("generated " + id) }
//! let checked = uuid_parse(id)?
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::{Env, Value};

use super::super::pure_native;

#[path = "uuid_gen.rs"]
mod generate;
#[path = "uuid_parse.rs"]
mod parse;

/// Register this group's built-ins.
///
/// Defines `uuid_v4`, `uuid_parse`, and `uuid_is_valid` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "uuid_v4",
        pure_native("uuid_v4", Some(0), |_args| {
            Ok(Value::Str(Rc::new(generate::v4())))
        }),
        false,
    );
    bindings.define(
        "uuid_parse",
        pure_native("uuid_parse", Some(1), |args| {
            Ok(result_value(parse::parse_arg(&args[0])))
        }),
        false,
    );
    bindings.define(
        "uuid_is_valid",
        pure_native("uuid_is_valid", Some(1), |args| {
            Ok(Value::Bool(parse::is_valid_arg(&args[0])))
        }),
        false,
    );
}
