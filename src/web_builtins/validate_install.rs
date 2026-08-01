//! Registration of the validation built-ins.
//!
//! Split from `validate.rs` so the entry point can carry the group's scope
//! documentation without pushing the file past the 50-line budget.

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::{Env, Value};

use super::super::super::pure_native;
use super::validate_fields::validate_fields;
use super::validate_phone::normalize_phone;
use super::validate_scan::{is_digits, is_email, is_slug};

/// Define `is_email`, `is_slug`, `is_digits`, `normalize_phone`, and
/// `validate_fields` in `env`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("is_email", predicate("is_email", is_email), false);
    bindings.define("is_slug", predicate("is_slug", is_slug), false);
    bindings.define("is_digits", predicate("is_digits", is_digits), false);
    bindings.define(
        "normalize_phone",
        pure_native("normalize_phone", Some(1), |args| {
            let text = str_arg(&args[0], "normalize_phone: text")?;
            Ok(result_value(
                normalize_phone(&text).map(|out| Value::Str(Rc::new(out))),
            ))
        }),
        false,
    );
    bindings.define(
        "validate_fields",
        pure_native("validate_fields", Some(2), |args| {
            Ok(result_value(validate_fields(&args[0], &args[1])))
        }),
        false,
    );
}

/// Build a one-argument predicate builtin that reports a bool.
///
/// A non-str argument is an error rather than `false`, so a type mistake in a
/// handler surfaces instead of masquerading as failed validation.
fn predicate(name: &'static str, check: fn(&str) -> bool) -> Value {
    pure_native(name, Some(1), move |args| {
        let text = str_arg(&args[0], name)?;
        Ok(Value::Bool(check(&text)))
    })
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
