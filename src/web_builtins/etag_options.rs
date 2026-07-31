//! Typed accessors for the `cache_control` options map.
//!
//! A missing key means "directive not requested", so it defaults rather than
//! erroring. A key present with the wrong type *is* an error: silently ignoring a
//! mistyped `no_store` would emit a cacheable header for a response the caller
//! meant to keep out of every cache.

use std::collections::HashMap;

use crate::value::Value;

/// Read a boolean directive, defaulting to false when absent or nil.
///
/// # Errors
///
/// Returns an error naming `key` when the value is present but not a bool.
pub(super) fn flag(opts: &HashMap<String, Value>, key: &str) -> Result<bool, String> {
    match opts.get(key) {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(format!(
            "cache_control: `{key}` must be bool, got {}",
            other.type_name()
        )),
    }
}

/// Read a seconds value, rejecting a negative lifetime.
///
/// # Errors
///
/// Returns an error naming `key` when the value is not an int, or is negative —
/// a negative freshness lifetime has no meaning and would be dropped by caches.
pub(super) fn seconds(opts: &HashMap<String, Value>, key: &str) -> Result<Option<i64>, String> {
    match opts.get(key) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Int(seconds)) if *seconds >= 0 => Ok(Some(*seconds)),
        Some(Value::Int(seconds)) => Err(format!(
            "cache_control: `{key}` must not be negative, got {seconds}"
        )),
        Some(other) => Err(format!(
            "cache_control: `{key}` must be int seconds, got {}",
            other.type_name()
        )),
    }
}
