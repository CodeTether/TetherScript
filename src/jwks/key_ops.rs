//! Reading a JWK `key_ops` array out of the in-tree JSON value model.
//!
//! One responsibility: turn the `key_ops` member into `Option<Vec<String>>`,
//! enforcing the array bound. Kept apart from [`crate::jwks::fields`] because that
//! file reads scalars and this one reads a bounded array.

use crate::jwks::limits::{MAX_FIELD_CHARS, MAX_KEY_OPS};
use crate::value::Value;

/// Read the optional `key_ops` array.
///
/// # Arguments
///
/// * `object` — The JWK, expected to be a JSON object.
/// * `label` — Locating name used in error text.
///
/// # Returns
///
/// `Some` with the owned operation names, or `None` when the member is absent or
/// JSON null. An empty array yields `Some(vec![])`, which
/// [`crate::jwks::usage::check`] then refuses — an explicitly empty `key_ops`
/// permits nothing, and that is different from having stated no restriction.
///
/// # Errors
///
/// Returns a named error when the member is not an array, when any element is not
/// a string or is over [`MAX_FIELD_CHARS`], or when the array holds more than
/// [`MAX_KEY_OPS`] entries.
///
/// # Panics
///
/// Does not panic: each `RefCell` borrow is released before its value is matched.
///
/// # Examples
///
/// Reached through the public surface, since this helper is crate-internal:
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
///
/// let set = JwkSet::parse(r#"{"keys":[{"kty":"RSA","key_ops":7}]}"#).unwrap();
/// assert!(set.skipped()[0].reason.contains("key_ops"));
/// ```
pub(crate) fn opt_key_ops(object: &Value, label: &str) -> Result<Option<Vec<String>>, String> {
    let Value::Map(members) = object else {
        return Err(format!(
            "{label}: expected a JSON object, got {}",
            object.type_name()
        ));
    };
    let member = members.borrow().get("key_ops").cloned();
    let items = match member {
        None | Some(Value::Nil) => return Ok(None),
        Some(Value::List(items)) => items.borrow().clone(),
        Some(other) => {
            return Err(format!(
                "{label}.key_ops must be an array, got {}",
                other.type_name()
            ));
        }
    };
    bounded(&items, label)
}

/// Enforce the count bound, then read every element.
fn bounded(items: &[Value], label: &str) -> Result<Option<Vec<String>>, String> {
    if items.len() > MAX_KEY_OPS {
        return Err(format!(
            "{label}.key_ops has {} entries; limit is {MAX_KEY_OPS}",
            items.len()
        ));
    }
    let mut ops = Vec::with_capacity(items.len());
    for item in items {
        ops.push(one_op(item, label)?);
    }
    Ok(Some(ops))
}

/// Read one `key_ops` element.
fn one_op(item: &Value, label: &str) -> Result<String, String> {
    match item {
        Value::Str(text) if text.len() <= MAX_FIELD_CHARS => Ok((**text).clone()),
        Value::Str(_) => Err(format!(
            "{label}.key_ops entry exceeds {MAX_FIELD_CHARS} bytes"
        )),
        other => Err(format!(
            "{label}.key_ops entries must be strings, got {}",
            other.type_name()
        )),
    }
}
