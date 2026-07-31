//! Compact-serialization splitting and header algorithm checks.
//!
//! # Security
//!
//! [`require_hs256`] is where the classic JWT vulnerability is closed. The
//! algorithm is chosen by the *verifier*, not read from the token: a token
//! claiming `{"alg":"none"}` or `{"alg":"RS256"}` is rejected outright rather
//! than dispatched on. Trusting the header would let an attacker pick an
//! algorithm the verifier never intended, including no signature at all.

use crate::json;
use crate::value::Value;

use super::jwt_base64url_decode::decode;

/// The only algorithm this implementation signs or verifies.
pub(super) const HS256: &str = "HS256";

/// Split a compact JWS into its three segments.
///
/// # Arguments
///
/// * `token` — Compact serialization, `header.payload.signature`.
///
/// # Returns
///
/// The three segments, still base64url-encoded.
///
/// # Errors
///
/// Returns a named error unless exactly three segments are present and none is
/// empty. A two-segment token is the unsecured JWS form and is never accepted.
pub(super) fn split(token: &str) -> Result<(&str, &str, &str), String> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(format!(
            "jwt: expected 3 dot-separated segments, got {}",
            token.split('.').count()
        ));
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() {
        return Err("jwt: token has an empty segment".into());
    }
    Ok((header, payload, signature))
}

/// Decode one base64url segment as a JSON value.
///
/// # Errors
///
/// Returns a named error when the segment is not valid base64url, not UTF-8, or
/// not valid JSON.
pub(super) fn decode_json(label: &str, segment: &str) -> Result<Value, String> {
    let bytes = decode(label, segment)?;
    let text = String::from_utf8(bytes).map_err(|_| format!("jwt: {label} is not valid UTF-8"))?;
    json::parse_str(&text).map_err(|error| format!("jwt: {label} is not valid JSON: {error}"))
}

/// Require that the header declares exactly HS256.
///
/// # Errors
///
/// Returns a named error when `alg` is missing, is not a string, or is anything
/// other than `HS256` — including `none`.
pub(super) fn require_hs256(header: &Value) -> Result<(), String> {
    let Value::Map(map) = header else {
        return Err("jwt: header must be a JSON object".into());
    };
    let algorithm = map.borrow().get("alg").cloned();
    match algorithm {
        Some(Value::Str(name)) if name.as_str() == HS256 => Ok(()),
        Some(Value::Str(name)) => Err(format!(
            "jwt: unsupported alg `{name}`; only {HS256} is accepted"
        )),
        Some(other) => Err(format!(
            "jwt: header `alg` must be str, got {}",
            other.type_name()
        )),
        None => Err("jwt: header is missing `alg`".into()),
    }
}
