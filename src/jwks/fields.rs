//! Typed reads of JSON object members, using the in-tree JSON value model.
//!
//! One responsibility: pull a member of a known type out of a [`Value::Map`] and
//! name the member when it is absent or the wrong type. Keeping this here means
//! no other JWKS file repeats a `Value::Map` match or a `RefCell` borrow.
//!
//! # What the in-tree parser gives us
//!
//! [`crate::json::parse_str`] returns a [`Value`] and reports failure as a plain
//! `String` ending in `at byte N`. A JSON object arrives as
//! `Value::Map(Rc<RefCell<HashMap<String, Value>>>)`, so member access needs a
//! runtime borrow; every read below clones the member out of that borrow before
//! matching, so no borrow guard outlives the call and a nested read cannot panic
//! on an already-borrowed cell.

use crate::jwks::limits::MAX_FIELD_CHARS;
use crate::value::Value;

/// Read an optional string member.
///
/// # Arguments
///
/// * `object` — Value expected to be a JSON object.
/// * `name` — Member name.
/// * `label` — Locating name used in error text, such as `jwks: keys[0]`.
///
/// # Returns
///
/// `Some` with the owned text, or `None` when the member is absent or JSON null.
/// An absent member and an explicit `null` are treated alike because issuers emit
/// both for "not set".
///
/// # Errors
///
/// Returns a named error when `object` is not a JSON object, when the member is
/// present but not a string, or when the text exceeds
/// [`MAX_FIELD_CHARS`](crate::jwks::limits::MAX_FIELD_CHARS).
///
/// # Panics
///
/// Does not panic: the `RefCell` borrow is released before matching.
///
/// # Examples
///
/// Exercised through the public surface, since this helper is crate-internal:
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
///
/// // `alg` absent and `alg: null` are both read as "not set".
/// let set = JwkSet::parse(r#"{"keys":[{"kty":"oct","kid":"s","alg":null}]}"#).unwrap();
/// assert_eq!(set.keys().len(), 0);
/// assert_eq!(set.skipped().len(), 1);
/// ```
pub(crate) fn opt_str(object: &Value, name: &str, label: &str) -> Result<Option<String>, String> {
    let Value::Map(members) = object else {
        return Err(format!(
            "{label}: expected a JSON object, got {}",
            object.type_name()
        ));
    };
    let member = members.borrow().get(name).cloned();
    match member {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Str(text)) if text.len() > MAX_FIELD_CHARS => Err(format!(
            "{label}.{name} is {} bytes; limit is {MAX_FIELD_CHARS}",
            text.len()
        )),
        Some(Value::Str(text)) => Ok(Some((*text).clone())),
        Some(other) => Err(format!(
            "{label}.{name} must be a string, got {}",
            other.type_name()
        )),
    }
}

/// Read a required string member.
///
/// # Arguments
///
/// * `object` — Value expected to be a JSON object.
/// * `name` — Member name.
/// * `label` — Locating name used in error text.
///
/// # Returns
///
/// The owned text.
///
/// # Errors
///
/// Returns a named error when the member is missing, null, not a string, or over
/// the field size limit.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
///
/// // A JWK with no `kty` at all is skipped, naming the missing member.
/// let set = JwkSet::parse(r#"{"keys":[{"kid":"a"}]}"#).unwrap();
/// assert!(set.skipped()[0].reason.contains("kty"));
/// ```
pub(crate) fn req_str(object: &Value, name: &str, label: &str) -> Result<String, String> {
    opt_str(object, name, label)?.ok_or_else(|| format!("{label} is missing `{name}`"))
}
