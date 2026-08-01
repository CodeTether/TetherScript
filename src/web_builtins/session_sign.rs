//! Signing and verification of session cookie values.
//!
//! The wire format is `base64url(json_payload) "." base64url(hmac_tag)`.
//!
//! # Why the tag covers the encoded payload
//!
//! The tag is computed over the encoded payload segment exactly as transmitted,
//! and verification re-tags that same received segment rather than re-serializing
//! the decoded map. `Value::Map` is a `HashMap`, so JSON key order is not stable;
//! re-serializing on the verify path would produce different bytes than the signer
//! wrote and would reject valid cookies intermittently. That class of bug presents
//! as random logouts and is very hard to diagnose from logs.

use std::rc::Rc;

use super::super::hmac::{constant_time_eq, hmac_sha256};
use super::session_base64url::{decode, encode};
use super::session_payload::{payload_of, split};
use crate::json;
use crate::value::Value;

/// Build a signed cookie value from a payload map.
///
/// # Arguments
///
/// * `payload` — Map of session fields. Readable by anyone holding the cookie.
/// * `secret` — Shared signing key.
///
/// # Returns
///
/// The signed value `payload.tag`, both segments unpadded base64url.
///
/// # Errors
///
/// Returns an error when `payload` is not a map or cannot be JSON-encoded.
pub(super) fn sign(payload: &Value, secret: &str) -> Result<Value, String> {
    if !matches!(payload, Value::Map(_)) {
        return Err(format!(
            "session_sign: payload must be map, got {}",
            payload.type_name()
        ));
    }
    let json = json::encode_to_string(payload)
        .map_err(|error| format!("session_sign: cannot encode payload: {error}"))?;
    let encoded = encode(json.as_bytes());
    let tag = encode(&hmac_sha256(secret.as_bytes(), encoded.as_bytes()));
    Ok(Value::Str(Rc::new(format!("{encoded}.{tag}"))))
}

/// Verify a signed cookie value and return its payload.
///
/// # Arguments
///
/// * `value` — Cookie value previously produced by [`sign`].
/// * `secret` — Shared signing key.
///
/// # Returns
///
/// The decoded payload map.
///
/// # Errors
///
/// Returns a named error for a missing separator, invalid base64url, a tag that
/// does not match, or a payload that is not JSON. Expiry is deliberately **not**
/// checked here, so a caller can tell a forged cookie from a merely stale one;
/// see `session_expired`.
pub(super) fn verify(value: &str, secret: &str) -> Result<Value, String> {
    let (encoded, presented) = split(value)?;
    let expected = hmac_sha256(secret.as_bytes(), encoded.as_bytes());
    let presented = decode("tag", presented)?;
    // Constant-time: never early-exit on the first differing byte.
    if !constant_time_eq(&expected, &presented) {
        return Err("session_verify: signature does not match".into());
    }
    payload_of(encoded)
}
