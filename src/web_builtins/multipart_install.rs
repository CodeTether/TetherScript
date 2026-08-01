//! Environment registration for the multipart group.
//!
//! Split from `multipart.rs` so the entry point stays documentation plus module
//! declarations, and the binding list has one obvious home.

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::{Env, Value};

use super::super::super::pure_native;
use super::{multipart_boundary, multipart_split, multipart_value};

/// Define `multipart_parse`, `multipart_field`, and `multipart_boundary`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "multipart_parse",
        pure_native("multipart_parse", Some(2), |args| {
            let body = str_arg(&args[0], "multipart_parse: body")?;
            let boundary = str_arg(&args[1], "multipart_parse: boundary")?;
            Ok(result_value(
                multipart_split::split(&body, &boundary).map(multipart_value::to_value),
            ))
        }),
        false,
    );
    bindings.define(
        "multipart_field",
        pure_native("multipart_field", Some(2), |args| {
            let name = str_arg(&args[1], "multipart_field: name")?;
            Ok(result_value(multipart_value::field(&args[0], &name)))
        }),
        false,
    );
    bindings.define(
        "multipart_boundary",
        pure_native("multipart_boundary", Some(1), |args| {
            let header = str_arg(&args[0], "multipart_boundary: content_type")?;
            Ok(result_value(
                multipart_boundary::boundary(&header).map(|text| Value::Str(Rc::new(text))),
            ))
        }),
        false,
    );
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
