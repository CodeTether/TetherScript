//! Segment splitting and payload decoding.
//!
//! Kept apart from [`super::session_sign`] so the authenticated path and the
//! decode-only path are never read as one blob: `payload_of` performs **no**
//! signature check and is reused by the deliberately-unverified builtin.

use std::rc::Rc;

use super::session_base64url::decode;
use crate::json;
use crate::value::Value;

/// Split a signed value into its payload and tag segments.
///
/// # Arguments
///
/// * `value` — Cookie value of the form `payload.tag`.
///
/// # Returns
///
/// The encoded payload segment and the encoded tag segment.
///
/// # Errors
///
/// Returns an error when the separator is missing, or when the value carries a
/// number of segments other than two.
pub(super) fn split(value: &str) -> Result<(&str, &str), String> {
    let count = value.split('.').count();
    if count != 2 {
        return Err(format!(
            "session_verify: expected 2 `.`-separated segments, found {count}"
        ));
    }
    value
        .split_once('.')
        .ok_or_else(|| "session_verify: value is missing the `.` separator".to_string())
}

/// Decode a payload segment into a map, without checking any signature.
///
/// # Arguments
///
/// * `encoded` — The base64url payload segment.
///
/// # Returns
///
/// The decoded payload map.
///
/// # Errors
///
/// Returns a named error when the segment is not base64url, not UTF-8, not JSON,
/// or decodes to something other than a map.
pub(super) fn payload_of(encoded: &str) -> Result<Value, String> {
    let bytes = decode("payload", encoded)?;
    let text =
        String::from_utf8(bytes).map_err(|_| "session: payload is not valid UTF-8".to_string())?;
    let parsed = json::parse(&Value::Str(Rc::new(text)))
        .map_err(|error| format!("session: payload is not valid JSON: {error}"))?;
    match parsed {
        Value::Map(_) => Ok(parsed),
        other => Err(format!(
            "session: payload must decode to a map, got {}",
            other.type_name()
        )),
    }
}
