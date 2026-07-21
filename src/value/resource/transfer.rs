//! Explicit ownership transfer into persistent resource storage.

mod aggregate;
mod scan;
mod unique;

use crate::value::Value;

/// Validate that every resource-owning value has a single live owner.
pub(crate) fn validate(value: &Value, operation: &str) -> Result<(), String> {
    scan::validate(value, operation)
}

/// Validate a transfer and clone the value for a retaining sink.
pub(crate) fn retained(value: &Value, operation: &str) -> Result<Value, String> {
    validate(value, operation)?;
    Ok(value.clone())
}
