//! Shared argument validation for HTTP and HTTPS listeners.

use crate::value::Value;

pub(super) fn port(value: &Value, builtin: &str) -> Result<u16, String> {
    match value {
        Value::Int(port) if *port > 0 && *port <= 65535 => Ok(*port as u16),
        Value::Int(port) => Err(format!("{builtin}: port {port} out of range")),
        other => Err(format!(
            "{builtin}: port must be int, got {}",
            other.type_name()
        )),
    }
}

pub(super) fn handler(value: &Value, builtin: &str) -> Result<(), String> {
    if matches!(value, Value::Fn(_) | Value::VmFn(_) | Value::Native(_)) {
        return Ok(());
    }
    Err(format!(
        "{builtin}: handler must be a function, got {}",
        value.type_name()
    ))
}

#[cfg(feature = "openssl-tls")]
pub(super) fn pem<'a>(value: &'a Value, builtin: &str, label: &str) -> Result<&'a str, String> {
    match value {
        Value::Str(value) if !value.is_empty() => Ok(value),
        Value::Str(_) => Err(format!("{builtin}: {label} PEM must not be empty")),
        other => Err(format!(
            "{builtin}: {label} PEM must be str, got {}",
            other.type_name()
        )),
    }
}
