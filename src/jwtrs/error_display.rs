//! `Display` and `Error` impls for [`JwtError`].
//!
//! One responsibility: dispatch between the three wording tables. Adding a
//! variant touches [`crate::jwtrs::error_shape`] or
//! [`crate::jwtrs::error_claims`]; rewording touches one `error_text_*` file;
//! neither touches this one.
//!
//! # Integration
//!
//! The integrator wires this with `mod error_display;`. It declares no public
//! items of its own.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::error::JwtError;
//! use tetherscript::jwtrs::error_claims::ClaimError;
//!
//! let err = JwtError::Claim(ClaimError::Expired { exp: 100, now: 300, skew: 60 });
//! let text = format!("{err}");
//! assert!(text.contains("expired at 100"));
//! assert!(text.contains("60s skew"));
//! ```

use std::fmt;

use crate::jwtrs::error::JwtError;
use crate::jwtrs::error_text_claims::claim_text;
use crate::jwtrs::error_text_roles::roles_text;
use crate::jwtrs::error_text_shape::shape_text;

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Shape(inner) => shape_text(inner),
            Self::Signature(detail) => {
                format!("jwtrs: signature verification failed: {detail}")
            }
            // Every `ClaimError` is covered by exactly one table; the fallback
            // exists only so adding a variant cannot panic.
            Self::Claim(inner) => claim_text(inner)
                .or_else(|| roles_text(inner))
                .unwrap_or_else(|| "jwtrs: claim validation failed".to_string()),
        };
        f.write_str(&text)
    }
}

impl std::error::Error for JwtError {}
