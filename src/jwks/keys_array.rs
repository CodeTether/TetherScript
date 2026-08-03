//! Extraction of the `keys` array from a parsed JWKS document.
//!
//! One responsibility: confirm the document shape and enforce the key-count bound.
//! Split from [`crate::jwks::document`] so that file owns the parse-and-partition
//! flow and this one owns the top-level shape rules.

use crate::jwks::error::JwksError;
use crate::jwks::limits::MAX_KEYS;
use crate::value::Value;

/// Extract the `keys` array.
///
/// # Arguments
///
/// * `document` — The JSON value the in-tree parser produced.
///
/// # Returns
///
/// The array elements, cloned out of the `RefCell` so no borrow guard outlives the
/// call. An empty array is valid and yields an empty vector: a realm with no
/// published keys is a realm no token verifies against, which is a selection
/// failure to report later, not a parse failure now.
///
/// # Errors
///
/// Returns [`JwksError::NotAnObject`] when the top level is not a JSON object,
/// [`JwksError::MissingKeys`] when there is no `keys` member (or it is null),
/// [`JwksError::KeysNotArray`] when it is present but not an array, and
/// [`JwksError::TooManyKeys`] when it holds more than [`MAX_KEYS`] entries.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn keys_array(document: &Value) -> Result<Vec<Value>, JwksError> {
    let Value::Map(members) = document else {
        return Err(JwksError::NotAnObject(document.type_name().to_string()));
    };
    let member = members.borrow().get("keys").cloned();
    let items = match member {
        None | Some(Value::Nil) => return Err(JwksError::MissingKeys),
        Some(Value::List(items)) => items.borrow().clone(),
        Some(other) => return Err(JwksError::KeysNotArray(other.type_name().to_string())),
    };
    if items.len() > MAX_KEYS {
        return Err(JwksError::TooManyKeys {
            count: items.len(),
            limit: MAX_KEYS,
        });
    }
    Ok(items)
}
