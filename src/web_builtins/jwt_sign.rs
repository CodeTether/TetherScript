//! HS256 signing and verification over the signing input.
//!
//! The signing input is `base64url(header) "." base64url(payload)`, per RFC 7515.

use std::rc::Rc;

use crate::json;
use crate::value::Value;

// The HMAC group re-exports both primitives `pub(crate)`, so this group reuses
// them instead of carrying a third copy of the construction. `jwt` and `hmac` are
// siblings under `web_builtins`, hence the two `super`s.
use super::super::hmac::{constant_time_eq, hmac_sha256};
use super::jwt_base64url::encode;
use super::jwt_base64url_decode::decode;
use super::jwt_header::{decode_json, require_hs256, split, HS256};

/// Fixed HS256 header. Emitted verbatim so the signing input is reproducible.
const HEADER_JSON: &str = r#"{"alg":"HS256","typ":"JWT"}"#;

/// Sign a claims map, returning the compact serialization.
///
/// # Arguments
///
/// * `claims` — Map encoded as the payload.
/// * `secret` — Shared HMAC key.
///
/// # Returns
///
/// `header.payload.signature`, every segment unpadded base64url.
///
/// # Errors
///
/// Returns a named error when `claims` is not a map or cannot be JSON-encoded.
pub(super) fn sign(claims: &Value, secret: &str) -> Result<Value, String> {
    if !matches!(claims, Value::Map(_)) {
        return Err(format!(
            "jwt_sign: claims must be map, got {}",
            claims.type_name()
        ));
    }
    let payload = json::encode_to_string(claims)
        .map_err(|error| format!("jwt_sign: cannot encode claims: {error}"))?;
    let signing_input = format!(
        "{}.{}",
        encode(HEADER_JSON.as_bytes()),
        encode(payload.as_bytes())
    );
    let signature = encode(&hmac_sha256(secret.as_bytes(), signing_input.as_bytes()));
    Ok(Value::Str(Rc::new(format!("{signing_input}.{signature}"))))
}

/// Verify a compact JWS and return its claims.
///
/// # Arguments
///
/// * `token` — Compact serialization to verify.
/// * `secret` — Shared HMAC key.
///
/// # Returns
///
/// The decoded payload map.
///
/// # Errors
///
/// Returns a named error for a bad segment count, invalid base64url, invalid
/// JSON, an `alg` other than `HS256` (including `none`), a signature mismatch,
/// an expired `exp`, or a future `nbf`. The signature is checked *before* the
/// claims, so an unauthenticated token never reaches claim handling.
pub(super) fn verify(token: &str, secret: &str) -> Result<Value, String> {
    let (header_segment, payload_segment, signature_segment) = split(token)?;
    require_hs256(&decode_json("header", header_segment)?)?;

    let signing_input = format!("{header_segment}.{payload_segment}");
    let expected = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let presented = decode("signature", signature_segment)?;
    // Constant-time: never early-exit on the first differing byte.
    if !constant_time_eq(&expected, &presented) {
        return Err(format!("jwt: {HS256} signature does not match"));
    }

    let claims = decode_json("payload", payload_segment)?;
    super::jwt_claims::validate(&claims)?;
    Ok(claims)
}

/// Decode the payload of a token **without** verifying its signature.
///
/// # Arguments
///
/// * `token` — Compact serialization to inspect.
///
/// # Returns
///
/// The decoded payload map. The value is untrusted: anyone can author it.
///
/// # Errors
///
/// Returns a named error for a bad segment count, invalid base64url, or invalid
/// JSON. Neither the signature nor `exp`/`nbf` is checked, which is why the
/// builtin is named `jwt_decode_unverified` rather than `jwt_decode`.
pub(super) fn decode_unverified(token: &str) -> Result<Value, String> {
    let (_, payload_segment, _) = split(token)?;
    decode_json("payload", payload_segment)
}
