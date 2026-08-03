//! Decoding the third segment.
//!
//! One responsibility: turn the signature segment into bytes. Trivially small, but
//! separate from [`crate::jwtrs::segment`] because the signature is *not* JSON —
//! it is an opaque octet string — so folding it into the JSON path would invite a
//! future reader to parse it.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::signature::decode_signature;
//!
//! assert_eq!(decode_signature("c2ln").unwrap().as_slice(), b"sig".as_slice());
//! // The signature obeys the same strict alphabet as every other segment.
//! assert!(decode_signature("c2l+").is_err());
//! ```

use crate::jwtrs::base64url_decode::decode;
use crate::jwtrs::error_shape::ShapeError;

/// Decode the signature segment.
///
/// # Arguments
///
/// * `segment` — The still-encoded third segment.
///
/// # Returns
///
/// The raw signature octets, passed to the verifier unmodified.
///
/// # Errors
///
/// [`ShapeError::Base64`] when the segment is not strict unpadded base64url.
///
/// # Panics
///
/// Does not panic.
pub fn decode_signature(segment: &str) -> Result<Vec<u8>, ShapeError> {
    decode("signature", segment).map_err(|reason| ShapeError::Base64 {
        segment: "signature",
        reason,
    })
}
