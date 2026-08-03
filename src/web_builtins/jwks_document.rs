//! JWKS document parsing: JSON text in, list of validated keys out.
//!
//! One responsibility: locate the `keys` array and normalize each element. The
//! per-key rules live in `super::jwks_key`.
//!
//! # Security
//!
//! Every entry must validate. A document with one good key and one 1024-bit key
//! is rejected wholesale rather than silently pruned, because "we quietly dropped
//! a key" and "the issuer rotated that key out" look identical to a caller and
//! only one of them is safe to ignore.

use std::cell::RefCell;
use std::rc::Rc;

use crate::json;
use crate::value::Value;

use super::jwks_key::normalize;

/// Parse a JWKS document.
///
/// # Arguments
///
/// * `text` — The JSON body of a JWKS endpoint response.
///
/// # Returns
///
/// A list of key maps in document order; see `super::jwks_key::normalize` for the
/// shape of each element. An empty `keys` array yields an empty list, which is
/// not an error: an issuer that has revoked everything is a valid state.
///
/// # Errors
///
/// Returns a named error when `text` is not valid JSON, is not an object, lacks a
/// `keys` array, or contains any key that fails validation. The error names the
/// offending index.
///
/// # Examples
///
/// ```tether
/// let keys = jwks_parse(certs_json).unwrap()
/// println(str(keys.len()))
/// println(str(jwks_parse("not json").is_err()))   // true
/// ```
pub(super) fn parse(text: &str) -> Result<Value, String> {
    let document = json::parse_str(text)
        .map_err(|error| format!("jwks: document is not valid JSON: {error}"))?;
    let Value::Map(fields) = &document else {
        return Err(format!(
            "jwks: document must be a JSON object, got {}",
            document.type_name()
        ));
    };
    // Cloned out of the borrow before matching, as `jwt_header.rs` does, so no
    // `RefCell` guard is alive while the per-key validation runs.
    let member = fields.borrow().get("keys").cloned();
    let entries = match member {
        Some(Value::List(items)) => items.borrow().clone(),
        Some(other) => {
            return Err(format!(
                "jwks: `keys` must be a list, got {}",
                other.type_name()
            ))
        }
        None => return Err("jwks: document is missing `keys`".into()),
    };
    let mut keys = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        keys.push(normalize(entry, index)?);
    }
    Ok(Value::List(Rc::new(RefCell::new(keys))))
}
