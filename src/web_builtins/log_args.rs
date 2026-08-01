//! Argument coercion for the logging built-ins.
//!
//! Kept separate so `log_install` stays a registration list. A wrong argument
//! type names the parameter rather than logging a placeholder, because a log call
//! that silently succeeded with the wrong payload would be worse than a failure.

use std::rc::Rc;

use super::{log_emit, log_level};
use crate::value::Value;

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Back `log_json(level, message, fields)`.
pub(super) fn log_json(args: &[Value]) -> Result<Value, String> {
    let level = str_arg(&args[0], "log_json: level")?;
    let message = str_arg(&args[1], "log_json: message")?;
    Ok(Value::Str(Rc::new(log_emit::emit(
        &level, &message, &args[2],
    )?)))
}

/// Back a fixed-level wrapper such as `log_info(message)`.
pub(super) fn log_at(level: &str, label: &str, args: &[Value]) -> Result<Value, String> {
    let message = str_arg(&args[0], label)?;
    Ok(Value::Str(Rc::new(log_emit::emit(
        level,
        &message,
        &Value::Nil,
    )?)))
}

/// Back `log_level_enabled(level)`.
pub(super) fn level_enabled(level: &Value) -> Result<Value, String> {
    let level = str_arg(level, "log_level_enabled: level")?;
    Ok(Value::Bool(log_level::enabled(
        &level,
        &log_emit::threshold(),
    )?))
}
