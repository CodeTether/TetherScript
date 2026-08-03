//! Normalization of one decoded JWK into a script-facing key map.
//!
//! One responsibility: take a single JSON object from a JWKS `keys` array and
//! produce the validated map a caller hands to an RSA verifier. Validation policy
//! lives in `super::jwks_rsa`; base64url decoding lives in
//! `super::jwks_base64url`. This file only wires them together and shapes the
//! result.
//!
//! # Shape of the returned map
//!
//! | Key | Type | Notes |
//! |---|---|---|
//! | `kid` | str | Key id; required, because selection depends on it |
//! | `kty` | str | Always `RSA`; anything else was already refused |
//! | `alg` | str or nil | Absent in some issuers' documents |
//! | `use` | str or nil | Typically `sig` |
//! | `modulus`, `n` | bytes | Big-endian RSA modulus, identical values |
//! | `exponent`, `e` | bytes | Big-endian RSA public exponent |
//! | `modulus_bits` | int | Significant bits, leading zeros excluded |

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::jwks_base64url::decode;
use super::jwks_field::{opt_str, req_str};
use super::jwks_rsa::{check_material, require_rsa};

/// Validate and normalize one JWK.
///
/// # Arguments
///
/// * `key` — One element of the JWKS `keys` array, already JSON-decoded.
/// * `index` — Position in the array, used to build a locating error label.
///
/// # Returns
///
/// The key map described in the module docs, with raw byte values for `n`/`e`.
///
/// # Errors
///
/// Returns a named error when the entry is not an object, is missing `kid`,
/// `kty`, `n`, or `e`, declares a non-RSA `kty`, carries a modulus under 2048
/// bits, has an empty exponent, or encodes `n`/`e` in anything other than strict
/// base64url.
///
/// # Examples
///
/// ```tether
/// let keys = jwks_parse(certs_json).unwrap()
/// println(keys[0].kty)                      // RSA
/// println(str(keys[0].modulus_bits >= 2048)) // true
/// ```
pub(super) fn normalize(key: &Value, index: usize) -> Result<Value, String> {
    let label = format!("jwks: keys[{index}]");
    let kty = req_str(key, "kty", &label)?;
    require_rsa(&kty, &label)?;
    let kid = req_str(key, "kid", &label)?;
    let modulus = decode(&format!("{label}.n"), &req_str(key, "n", &label)?)?;
    let exponent = decode(&format!("{label}.e"), &req_str(key, "e", &label)?)?;
    let bits = check_material(&modulus, &exponent, &label)?;

    let mut out = HashMap::new();
    out.insert("kid".into(), Value::Str(Rc::new(kid)));
    out.insert("kty".into(), Value::Str(Rc::new(kty)));
    out.insert("alg".into(), optional(opt_str(key, "alg", &label)?));
    out.insert("use".into(), optional(opt_str(key, "use", &label)?));
    out.insert("modulus_bits".into(), Value::Int(bits));
    insert_bytes(&mut out, "modulus", "n", modulus);
    insert_bytes(&mut out, "exponent", "e", exponent);
    Ok(Value::Map(Rc::new(RefCell::new(out))))
}

/// Represent an absent optional member as `nil` rather than omitting the key, so
/// a script reading `key.alg` always sees a defined field.
fn optional(text: Option<String>) -> Value {
    match text {
        Some(value) => Value::Str(Rc::new(value)),
        None => Value::Nil,
    }
}

/// Store one byte string under both its descriptive and JWK-spelled names.
///
/// The two names share one `Rc`, so they are the same buffer and cannot drift.
fn insert_bytes(out: &mut HashMap<String, Value>, long: &str, short: &str, bytes: Vec<u8>) {
    let value = Value::Bytes(Rc::new(RefCell::new(bytes)));
    out.insert(long.into(), value.clone());
    out.insert(short.into(), value);
}
