//! The validated policy representation, and reading it back out of a map.
//!
//! `cors_policy` hands the script a plain map so it can be printed, stored, or
//! passed between modules like any other value. `cors_preflight` and
//! `cors_headers` then read that map back into [`Policy`]. Reading it back —
//! rather than trusting the map's shape — keeps the per-request path honest even
//! if a script hand-builds a map or mutates one field of a real policy.

use std::collections::HashMap;

use super::cors_args::{map_arg, string_list};
use super::cors_fields as key;
use crate::value::Value;

/// A validated CORS policy.
pub(super) struct Policy {
    /// True when `origins` was the string `"*"`. Mutually exclusive with
    /// `credentials`; see `cors_config`.
    pub(super) wildcard: bool,
    /// Exact origins, compared byte for byte. Empty when `wildcard`.
    pub(super) origins: Vec<String>,
    /// Allowed methods, upper-cased.
    pub(super) methods: Vec<String>,
    /// Allowed request header names, lower-cased.
    pub(super) headers: Vec<String>,
    /// Response header names exposed to script, lower-cased.
    pub(super) expose: Vec<String>,
    /// Whether cookies and `Authorization` may accompany the request.
    pub(super) credentials: bool,
    /// Preflight cache lifetime in seconds, when requested.
    pub(super) max_age: Option<i64>,
}

/// Read a policy map produced by `cors_policy`.
///
/// # Arguments
///
/// * `value` — The policy map.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The [`Policy`] the per-request built-ins operate on.
///
/// # Errors
///
/// Returns an error when `value` is not a map or any field has the wrong type.
pub(super) fn read(value: &Value, label: &str) -> Result<Policy, String> {
    let map = map_arg(value, &format!("{label}: policy"))?;
    Ok(Policy {
        wildcard: flag(&map, key::WILDCARD, label)?,
        origins: list(&map, key::ORIGINS, label)?,
        methods: list(&map, key::METHODS, label)?,
        headers: list(&map, key::HEADERS, label)?,
        expose: list(&map, key::EXPOSE, label)?,
        credentials: flag(&map, key::CREDENTIALS, label)?,
        max_age: match map.get(key::MAX_AGE) {
            Some(Value::Int(seconds)) => Some(*seconds),
            _ => None,
        },
    })
}

/// Read one list field, defaulting to empty when absent.
fn list(map: &HashMap<String, Value>, name: &str, label: &str) -> Result<Vec<String>, String> {
    match map.get(name) {
        None | Some(Value::Nil) => Ok(Vec::new()),
        Some(value) => string_list(value, &format!("{label}: policy `{name}`")),
    }
}

/// Read one bool field, defaulting to false when absent.
fn flag(map: &HashMap<String, Value>, name: &str, label: &str) -> Result<bool, String> {
    match map.get(name) {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(format!(
            "{label}: policy `{name}` must be bool, got {}",
            other.type_name()
        )),
    }
}
