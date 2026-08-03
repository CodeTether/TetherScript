//! Map-field reading helpers shared by the JWKS concerns.
//!
//! One responsibility: pull a typed member out of a decoded JSON object and name
//! the field when it is absent or the wrong type. Keeping this separate means no
//! other file in the group repeats a `Value::Map` match.

use crate::value::Value;

/// Read an optional string member.
///
/// # Arguments
///
/// * `map` — Value expected to be a JSON object.
/// * `name` — Member name.
/// * `label` — Qualified name used in error text.
///
/// # Returns
///
/// `Some` with the owned text, or `None` when the member is absent or JSON null.
///
/// # Errors
///
/// Returns a named error when `map` is not an object, or when the member is
/// present but not a string.
///
/// # Examples
///
/// ```tether
/// // A JWK without `use` still parses; the member is simply absent.
/// println(str(jwks_parse("{\"keys\":[]}").unwrap().len()))   // 0
/// ```
pub(super) fn opt_str(map: &Value, name: &str, label: &str) -> Result<Option<String>, String> {
    let Value::Map(fields) = map else {
        return Err(format!(
            "{label}: expected a JSON object, got {}",
            map.type_name()
        ));
    };
    // Cloned out of the borrow before matching so no guard outlives this call.
    let member = fields.borrow().get(name).cloned();
    match member {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Str(text)) => Ok(Some((*text).clone())),
        Some(other) => Err(format!(
            "{label}.{name} must be str, got {}",
            other.type_name()
        )),
    }
}

/// Read a required string member.
///
/// # Arguments
///
/// * `map` — Value expected to be a JSON object.
/// * `name` — Member name.
/// * `label` — Qualified name used in error text.
///
/// # Returns
///
/// The owned text.
///
/// # Errors
///
/// Returns a named error when the member is missing, null, or not a string.
///
/// # Examples
///
/// ```tether
/// println(str(jwks_parse("{\"keys\":[{}]}").is_err()))   // true
/// ```
pub(super) fn req_str(map: &Value, name: &str, label: &str) -> Result<String, String> {
    opt_str(map, name, label)?.ok_or_else(|| format!("{label} is missing `{name}`"))
}
