//! Unverified claim inspection.
//!
//! Split from [`super::csrf_sign`] to keep the signing path and the deliberately
//! unauthenticated path in separate files, so no reader mistakes one for the other.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::csrf_parse::parse;

/// Build the claims map from an already-decoded payload.
///
/// # Arguments
///
/// * `text` — Decoded payload text.
///
/// # Returns
///
/// A map of `nonce`, `iat`, and `exp`. These values are untrusted: the signature
/// was not examined, so anyone could have authored them.
///
/// # Errors
///
/// Returns an error when the payload does not parse.
pub(super) fn claims_map(text: &str) -> Result<Value, String> {
    let parsed = parse(text)?;
    let mut map = HashMap::new();
    map.insert("nonce".to_string(), Value::Str(Rc::new(parsed.nonce)));
    map.insert("iat".to_string(), Value::Int(parsed.issued_at));
    map.insert("exp".to_string(), Value::Int(parsed.expires_at));
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}
