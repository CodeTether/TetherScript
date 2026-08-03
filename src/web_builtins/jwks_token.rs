//! Compact-serialization splitting and unverified header decoding.
//!
//! One responsibility: turn token text into its three segments and, on request,
//! into the decoded header object. No signature and no claim is examined here.
//!
//! # Security
//!
//! A two-segment token is refused. Two segments is the unsecured JWS form from
//! RFC 7515 §6, i.e. a token with no signature at all, and there is no context in
//! which this group should hand one onward. An empty segment is refused for the
//! same reason: an empty signature segment is `alg: none` wearing a hat.

use crate::json;
use crate::value::Value;

use super::jwks_base64url::decode;

/// Split a compact JWS into its three encoded segments.
///
/// # Arguments
///
/// * `label` — Built-in name used in error text.
/// * `token` — Compact serialization, `header.payload.signature`.
///
/// # Returns
///
/// The three still-encoded segments.
///
/// # Errors
///
/// Returns a named error unless there are exactly three segments and none is
/// empty. The error reports the count that was actually found.
///
/// # Examples
///
/// ```tether
/// println(str(jwt_header("only.two").is_err()))   // true
/// ```
pub(super) fn split<'a>(
    label: &str,
    token: &'a str,
) -> Result<(&'a str, &'a str, &'a str), String> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(format!(
            "{label}: expected 3 dot-separated segments, got {}",
            token.split('.').count()
        ));
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() {
        return Err(format!("{label}: token has an empty segment"));
    }
    Ok((header, payload, signature))
}

/// Decode the header segment as a JSON object, verifying nothing.
///
/// # Arguments
///
/// * `label` — Built-in name used in error text.
/// * `token` — Compact serialization to inspect.
///
/// # Returns
///
/// The decoded header map. **Untrusted**: see `super::jwks_parts::header`.
///
/// # Errors
///
/// Returns a named error for a bad segment count, invalid base64url, non-UTF-8
/// bytes, malformed JSON, or a header that decodes to a JSON value other than an
/// object.
///
/// # Examples
///
/// ```tether
/// println(str(jwt_header("eyJhbGciOiA.e30.AA").is_err()))   // true
/// ```
pub(super) fn header_object(label: &str, token: &str) -> Result<Value, String> {
    let (header, _, _) = split(label, token)?;
    let bytes = decode(&format!("{label}: header"), header)?;
    let text =
        String::from_utf8(bytes).map_err(|_| format!("{label}: header is not valid UTF-8"))?;
    let value = json::parse_str(&text)
        .map_err(|error| format!("{label}: header is not valid JSON: {error}"))?;
    if !matches!(value, Value::Map(_)) {
        return Err(format!(
            "{label}: header must be a JSON object, got {}",
            value.type_name()
        ));
    }
    Ok(value)
}
