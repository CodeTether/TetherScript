//! Decoding one JWS segment into a JSON object.
//!
//! One responsibility: base64url bytes → UTF-8 → JSON → *object*. Composed by both
//! the header and the payload paths so neither repeats the four-step ladder.
//!
//! # Reuse of the in-tree JSON parser
//!
//! Parsing goes through `crate::json`, the dependency-free parser this repository
//! already ships and the one the HS256 group uses. No JSON code is written here.
//!
//! # Security: a non-object payload is refused
//!
//! `crate::json` will happily parse `"[1,2,3]"`, `"7"`, `"null"`, or `"\"hi\""`.
//! None of those is a JWT payload: RFC 7519 §3 requires a JSON object. Every
//! downstream accessor expects a map, and code that shrugs at a non-map tends to
//! treat "no claims at all" as "no claims failed", which is precisely backwards.
//! So the shape is checked once, here, and everything downstream can rely on it.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::segment::decode_object;
//!
//! let good = encode(br#"{"sub":"user-1"}"#);
//! assert!(decode_object("payload", &good).is_ok());
//!
//! // A JSON array is valid JSON and an invalid payload.
//! let array = encode(b"[1,2,3]");
//! assert!(decode_object("payload", &array).is_err());
//! ```

use std::collections::HashMap;

use crate::json;
use crate::jwtrs::base64url_decode::decode;
use crate::jwtrs::error_shape::ShapeError;
use crate::value::Value;

/// Decode a segment into an owned JSON object map.
///
/// # Arguments
///
/// * `label` — `"header"` or `"payload"`, used in error text.
/// * `segment` — The still-encoded segment.
///
/// # Returns
///
/// The members, cloned out of the parser's `RefCell` so no borrow guard outlives
/// the call.
///
/// # Errors
///
/// [`ShapeError::Base64`], [`ShapeError::NotUtf8`], [`ShapeError::MalformedJson`],
/// or [`ShapeError::NotAnObject`], in that order.
///
/// # Panics
///
/// Does not panic.
pub fn decode_object(
    label: &'static str,
    segment: &str,
) -> Result<HashMap<String, Value>, ShapeError> {
    let bytes = decode(label, segment).map_err(|reason| ShapeError::Base64 {
        segment: label,
        reason,
    })?;
    let text = String::from_utf8(bytes).map_err(|_| ShapeError::NotUtf8(label))?;
    let parsed = json::parse_str(&text).map_err(|detail| ShapeError::MalformedJson {
        segment: label,
        detail,
    })?;
    match parsed {
        Value::Map(members) => Ok(members.borrow().clone()),
        other => Err(ShapeError::NotAnObject {
            segment: label,
            found: other.type_name(),
        }),
    }
}
