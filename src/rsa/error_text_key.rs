//! # Error text for key and signature rejections
//!
//! One responsibility: render the [`RsaError`] variants that are decided
//! *before* any padding is inspected — key validation and the two checks on the
//! signature integer itself.
//!
//! Splitting the rendering by decision stage keeps each file inside the
//! 50-line budget and mirrors the order the checks actually run in.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::RsaError;
//!
//! let text = format!("{}", RsaError::ModulusEven);
//! assert!(text.contains("even"));
//! ```

use super::error::RsaError;
use super::key_check::MIN_MODULUS_BYTES;

/// Render a key-level or signature-level rejection.
///
/// # Arguments
///
/// * `err` — the error to describe.
///
/// # Returns
///
/// `Some(text)` for the variants this stage owns, `None` for padding and digest
/// variants, which `super::error_text_padding` renders instead.
pub(super) fn key_text(err: &RsaError) -> Option<String> {
    Some(match err {
        RsaError::ModulusTooSmall { bytes } => format!(
            "rsa: modulus is {bytes} bytes; at least {MIN_MODULUS_BYTES} (2048-bit) required"
        ),
        RsaError::ModulusEven => {
            "rsa: modulus is even, so it is not a product of odd primes".to_string()
        }
        RsaError::ExponentTooSmall => {
            "rsa: public exponent must be at least 2; 0 and 1 verify anything".to_string()
        }
        RsaError::SignatureLength { got, expected } => {
            format!("rsa: signature is {got} bytes but the modulus is {expected} bytes")
        }
        RsaError::SignatureOutOfRange => {
            "rsa: signature integer is not less than the modulus".to_string()
        }
        RsaError::BigInt(inner) => format!("rsa: arithmetic failure: {inner}"),
        _ => return None,
    })
}
