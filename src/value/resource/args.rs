//! Typed argument conversion for resource factories and methods.

use crate::value::Value;

pub(super) fn string(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label}: expected str, got {}", other.type_name())),
    }
}

pub(super) fn bytes(value: &Value, label: &str) -> Result<Vec<u8>, String> {
    match value {
        Value::Str(text) => Ok(text.as_bytes().to_vec()),
        Value::Bytes(bytes) => Ok(bytes.borrow().clone()),
        other => Err(format!(
            "{label}: expected str or bytes, got {}",
            other.type_name()
        )),
    }
}

pub(super) fn usize(value: &Value, label: &str) -> Result<usize, String> {
    match value {
        Value::Int(number) if *number >= 0 => usize::try_from(*number)
            .map_err(|_| format!("{label}: {number} does not fit in host memory size")),
        Value::Int(number) => Err(format!("{label}: expected non-negative int, got {number}")),
        other => Err(format!("{label}: expected int, got {}", other.type_name())),
    }
}

pub(super) fn u64(value: &Value, label: &str) -> Result<u64, String> {
    match value {
        Value::Int(number) if *number >= 0 => Ok(*number as u64),
        Value::Int(number) => Err(format!("{label}: expected non-negative int, got {number}")),
        other => Err(format!("{label}: expected int, got {}", other.type_name())),
    }
}

pub(super) fn strings(value: &Value, label: &str) -> Result<Vec<String>, String> {
    let Value::List(values) = value else {
        return Err(format!("{label}: expected list, got {}", value.type_name()));
    };
    values
        .borrow()
        .iter()
        .map(|value| string(value, label))
        .collect()
}
