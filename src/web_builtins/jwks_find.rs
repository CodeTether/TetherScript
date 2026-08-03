//! Key selection by `kid`.
//!
//! One responsibility: given the list `jwks_parse` produced and a key id, return
//! the single matching key.
//!
//! # Security
//!
//! The `kid` a caller passes in normally comes from `jwt_header`, which is
//! unverified — so `kid` is attacker-chosen. That is safe *here* because a lookup
//! can only ever return a key the issuer published: an attacker can steer which
//! trusted key is tried, but cannot introduce an untrusted one. What would be
//! unsafe is falling back to "any key" on a miss, so a miss is a hard error.

use crate::value::Value;

use super::jwks_field::opt_str;

/// Find the key whose `kid` matches.
///
/// # Arguments
///
/// * `keys` — List returned by `jwks_parse`.
/// * `kid` — Key id to select, typically read from an unverified header.
///
/// # Returns
///
/// The matching key map, cloned by handle so the caller shares the same bytes.
///
/// # Errors
///
/// Returns a named error when `keys` is not a list, when an element is not a key
/// map, or when no element carries that `kid`. The absent case names the `kid`
/// that was requested and the ones that were available, because a `kid` mismatch
/// after a key rotation is the most common cause and is otherwise invisible.
///
/// # Examples
///
/// ```tether
/// let keys = jwks_parse(certs_json).unwrap()
/// println(jwks_find(keys, "key-a").unwrap().kid)     // key-a
/// println(str(jwks_find(keys, "nope").is_err()))     // true
/// ```
pub(super) fn find(keys: &Value, kid: &str) -> Result<Value, String> {
    let Value::List(items) = keys else {
        return Err(format!(
            "jwks_find: keys must be a list, got {}",
            keys.type_name()
        ));
    };
    let items = items.borrow();
    let mut available = Vec::with_capacity(items.len());
    for key in items.iter() {
        match opt_str(key, "kid", "jwks_find: key")? {
            Some(found) if found == kid => return Ok(key.clone()),
            Some(found) => available.push(found),
            None => available.push("<missing>".into()),
        }
    }
    Err(format!(
        "jwks_find: no key with kid `{kid}`; available: [{}]",
        available.join(", ")
    ))
}
