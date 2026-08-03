//! # RSA verification error taxonomy
//!
//! One responsibility: name every reason this crate refuses an RSA PKCS#1 v1.5
//! signature. Rendering lives in `super::error_display`; the checks that raise
//! these live in `super::key_check`, `super::pkcs1`, and `super::verify`.
//!
//! ## Why the taxonomy is this fine-grained
//!
//! Each variant corresponds to one *specific* forgery or misconfiguration class,
//! and there is a negative test per variant in `tests/rsa_pkcs1.rs` and
//! `tests/rsa_verify.rs`. Collapsing them into one opaque "invalid signature"
//! would make it impossible to prove by test that a given Bleichenbacher-style
//! relaxation is actually rejected rather than accidentally unreachable.
//!
//! Callers exposed to untrusted input should still surface a single generic
//! failure to the remote peer; the detail here is for logs and tests.
//!
//! ## Integration
//!
//! The integrator wires this with `mod error;` plus `pub use error::RsaError;`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::RsaError;
//!
//! let err = RsaError::PaddingRunTooShort { len: 3 };
//! assert!(format!("{err}").contains("0xff"));
//! ```

use crate::bigint::BigUintError;

/// Why an RSA public key was rejected, or why a signature failed to verify.
///
/// # Examples
///
/// ```rust
/// use tetherscript::rsa::RsaError;
///
/// let err = RsaError::ExponentTooSmall;
/// match err {
///     RsaError::ExponentTooSmall => {}
///     other => panic!("unexpected {other:?}"),
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsaError {
    /// Modulus shorter than 256 octets, i.e. under the 2048-bit floor.
    ModulusTooSmall {
        /// Significant octet length of the supplied modulus.
        bytes: usize,
    },
    /// Modulus is even, so it cannot be a product of two odd primes.
    ModulusEven,
    /// Public exponent is 0 or 1, both of which make verification vacuous.
    ExponentTooSmall,
    /// Signature octet length differs from the modulus octet length `k`.
    SignatureLength {
        /// Length of the supplied signature.
        got: usize,
        /// Required length, equal to the modulus octet length.
        expected: usize,
    },
    /// Signature integer is greater than or equal to the modulus.
    SignatureOutOfRange,
    /// `k` is too small to hold `0x00 0x01`, eight padding octets, the
    /// separator, and the DER `DigestInfo`.
    EncodingTooShort {
        /// Modulus octet length.
        modulus_bytes: usize,
        /// Minimum octet length the chosen digest algorithm requires.
        needed: usize,
    },
    /// The encoded message does not begin with `0x00 0x01`.
    LeadingBytes {
        /// First octet found.
        first: u8,
        /// Second octet found.
        second: u8,
    },
    /// The run of `0xFF` padding octets was shorter than eight.
    PaddingRunTooShort {
        /// Number of `0xFF` octets actually present.
        len: usize,
    },
    /// The `0xFF` run was not terminated by a `0x00` separator octet.
    MissingSeparator,
    /// The octets after the separator are not exactly one `DigestInfo` long.
    DigestInfoLength {
        /// Length required by the claimed digest algorithm.
        expected: usize,
        /// Length actually available after the separator.
        found: usize,
    },
    /// The DER `DigestInfo` prefix names a different hash than was claimed.
    DigestInfoMismatch,
    /// The recovered digest differs from the expected digest.
    DigestMismatch,
    /// The supplied digest length does not match the digest algorithm.
    DigestLength {
        /// Length the algorithm produces.
        expected: usize,
        /// Length actually supplied by the caller.
        found: usize,
    },
    /// An arithmetic step failed; see [`BigUintError`].
    BigInt(BigUintError),
}
