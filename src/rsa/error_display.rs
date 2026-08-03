//! # `Display` / `Error` / `From` impls for [`RsaError`]
//!
//! One responsibility: attach the standard error traits to [`RsaError`]. The
//! per-variant wording lives in `super::error_text_key` and
//! `super::error_text_padding`; this file only dispatches between them.
//!
//! ## Integration
//!
//! The integrator wires this with `mod error_display;`. It declares no public
//! items of its own.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUintError;
//! use tetherscript::rsa::RsaError;
//!
//! // `?` on a bigint call inside a verification path converts automatically.
//! let err: RsaError = BigUintError::DivideByZero.into();
//! assert_eq!(err, RsaError::BigInt(BigUintError::DivideByZero));
//! assert!(format!("{err}").starts_with("rsa: "));
//! ```

use std::fmt;

use crate::bigint::BigUintError;

use super::error::RsaError;
use super::error_text_key::key_text;
use super::error_text_padding::padding_text;

impl fmt::Display for RsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Every variant is covered by exactly one of the two tables; the
        // fallback exists only so this cannot panic if a variant is added.
        let text = key_text(self)
            .or_else(|| padding_text(self))
            .unwrap_or_else(|| "rsa: signature verification failed".to_string());
        f.write_str(&text)
    }
}

impl std::error::Error for RsaError {}

impl From<BigUintError> for RsaError {
    fn from(inner: BigUintError) -> Self {
        Self::BigInt(inner)
    }
}
