//! The single error type callers see, and the pipeline order encoded in it.
//!
//! One responsibility: sum the three stages of RS256 validation into one type.
//! The per-stage variant sets live in [`crate::jwtrs::error_shape`] and
//! [`crate::jwtrs::error_claims`]; the wording lives in
//! [`crate::jwtrs::error_display`].
//!
//! # The type *is* the pipeline
//!
//! ```text
//!   Shape(ShapeError)          decided on wholly untrusted bytes
//!       │
//!   Signature(String)          decided by the SignatureVerifier
//!       │
//!   Claim(ClaimError)          decided only on an authenticated payload
//! ```
//!
//! Reading the variants top to bottom reads the order the checks run in, and a
//! `Claim` variant can only be reached after `Signature` succeeded — see
//! [`crate::jwtrs::authenticated`] for why that is a compile-time fact rather
//! than a convention.
//!
//! # Deliberately *not* in here
//!
//! There is no variant for "key not found" or "modulus too small". Key selection
//! and RSA arithmetic sit behind
//! [`SignatureVerifier`](crate::jwtrs::verifier::SignatureVerifier) and surface
//! as the opaque [`JwtError::Signature`] payload, because this module owns claim
//! validation only.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::error::JwtError;
//! use tetherscript::jwtrs::error_shape::ShapeError;
//!
//! let err: JwtError = ShapeError::AlgNone.into();
//! assert_eq!(err, JwtError::Shape(ShapeError::AlgNone));
//! assert!(format!("{err}").starts_with("jwtrs: "));
//! ```

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::error_shape::ShapeError;

/// A named RS256 validation failure, tagged with the stage that rejected it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtError {
    /// Rejected before the signature was checked; see [`ShapeError`].
    Shape(ShapeError),
    /// The signature verifier refused the signature; its own message is carried.
    Signature(String),
    /// Rejected after the signature was accepted; see [`ClaimError`].
    Claim(ClaimError),
}

impl From<ShapeError> for JwtError {
    fn from(inner: ShapeError) -> Self {
        Self::Shape(inner)
    }
}

impl From<ClaimError> for JwtError {
    fn from(inner: ClaimError) -> Self {
        Self::Claim(inner)
    }
}
