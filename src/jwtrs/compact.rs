//! Compact-serialization splitting.
//!
//! One responsibility: turn a token string into its three encoded segments plus
//! the exact signing input, or refuse it. No decoding, no JSON, no claims.
//!
//! # Security: the signing input is a slice, never a rebuild
//!
//! [`Parts::signing_input`] is a borrowed slice of the *original* token text, not
//! `format!("{header}.{payload}")`. Re-encoding is how signature checks get
//! silently detached from the bytes the issuer actually signed: any normalisation
//! — a re-encoded segment, a stripped character — would verify a string the signer
//! never saw. Slicing makes that class of bug unrepresentable.
//!
//! # Why a two-segment token is refused
//!
//! `header.payload` with no third segment is the unsecured JWS form of RFC 7515
//! §A.5. It is a *valid* JWS that carries no signature at all, so accepting it
//! would accept anything.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::compact::Parts;
//!
//! let parts = Parts::split("aGVhZGVy.cGF5bG9hZA.c2ln").unwrap();
//! assert_eq!(parts.header_b64, "aGVhZGVy");
//! assert_eq!(parts.signing_input, "aGVhZGVy.cGF5bG9hZA");
//!
//! assert!(Parts::split("aGVhZGVy.cGF5bG9hZA").is_err());       // 2 segments
//! assert!(Parts::split("a.b.c.d").is_err());                    // 4 segments
//! assert!(Parts::split("aGVhZGVy..c2ln").is_err());             // empty payload
//! ```

use crate::jwtrs::error_shape::ShapeError;
use crate::jwtrs::limits::MAX_TOKEN_BYTES;

/// The three encoded segments of a compact JWS, plus its signing input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts<'token> {
    /// First segment, still base64url-encoded.
    pub header_b64: &'token str,
    /// Second segment, still base64url-encoded.
    pub payload_b64: &'token str,
    /// Third segment, still base64url-encoded.
    pub signature_b64: &'token str,
    /// Exactly `header_b64 || "." || payload_b64`, sliced from the original token.
    pub signing_input: &'token str,
}

impl<'token> Parts<'token> {
    /// Split a compact JWS.
    ///
    /// # Arguments
    ///
    /// * `token` — The compact serialization, without any `Bearer ` prefix.
    ///
    /// # Returns
    ///
    /// The three encoded segments and the signing input, all borrowed from `token`.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::TokenTooLarge`] past
    /// [`MAX_TOKEN_BYTES`], [`ShapeError::WrongSegmentCount`] unless there are
    /// exactly three segments, and [`ShapeError::EmptySegment`] when any segment is
    /// empty — including the signature, since an empty signature is the unsecured
    /// form wearing three segments.
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn split(token: &'token str) -> Result<Self, ShapeError> {
        if token.len() > MAX_TOKEN_BYTES {
            return Err(ShapeError::TokenTooLarge {
                bytes: token.len(),
                limit: MAX_TOKEN_BYTES,
            });
        }
        let count = token.split('.').count();
        if count != 3 {
            return Err(ShapeError::WrongSegmentCount(count));
        }
        // Exactly three segments, so the last dot separates the signature and the
        // remainder is byte-identical to what the signer signed.
        let (signing_input, signature_b64) = token
            .rsplit_once('.')
            .ok_or(ShapeError::WrongSegmentCount(count))?;
        let (header_b64, payload_b64) = signing_input
            .split_once('.')
            .ok_or(ShapeError::WrongSegmentCount(count))?;
        for (label, segment) in [
            ("header", header_b64),
            ("payload", payload_b64),
            ("signature", signature_b64),
        ] {
            if segment.is_empty() {
                return Err(ShapeError::EmptySegment(label));
            }
        }
        Ok(Self {
            header_b64,
            payload_b64,
            signature_b64,
            signing_input,
        })
    }
}
