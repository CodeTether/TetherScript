//! Argument coercion and result wrapping for the template built-ins.
//!
//! Split from `template.rs` so that file holds documentation, submodule
//! declarations, and registration only, and stays within the line budget.

use std::rc::Rc;

use crate::system::result_value;
use crate::value::Value;

// One level deeper than `template.rs`, hence the third `super`.
use super::super::super::pure_native;

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Wrap a fallible render as a tetherscript `Result`.
pub(super) fn wrap(result: Result<String, String>) -> Value {
    result_value(result.map(|text| Value::Str(Rc::new(text))))
}

/// Build a pure native from a name, arity, and body.
pub(super) fn native<F>(name: &str, arity: usize, func: F) -> Value
where
    F: Fn(&[Value]) -> Result<Value, String> + 'static,
{
    pure_native(name, Some(arity), func)
}
