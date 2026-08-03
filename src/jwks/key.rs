//! The validated RSA public key this module hands to a signature verifier.
//!
//! One responsibility: hold already-validated key material. Construction happens
//! in [`crate::jwks::parse_key`]; nothing here re-checks, because a value of this
//! type existing *is* the claim that validation passed.
//!
//! # Contract for the verifier
//!
//! An [`RsaPublicKey`] guarantees all of the following, so a verifier may rely on
//! them without re-testing:
//!
//! * `modulus` and `exponent` are **big-endian**, **minimal** (no leading zero
//!   bytes), and non-empty.
//! * `modulus_bits` is between
//!   [`MIN_MODULUS_BITS`](crate::jwks::limits::MIN_MODULUS_BITS) and
//!   `MAX_MODULUS_BYTES * 8`, and the modulus is odd.
//! * `exponent` is odd and at least 3 when read as an integer.
//! * The key is usable for signature verification: its `use` was absent or `sig`,
//!   and its `key_ops`, if present, contained `verify`.
//! * If `alg` is `Some`, it matched the algorithm the caller requested.

use crate::jwks::alg::SigAlg;

/// A validated RSA public key selected from a JWKS document.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
/// use tetherscript::jwks::alg::SigAlg;
///
/// let set = JwkSet::parse(tetherscript::jwks::keyset::EXAMPLE_JWKS).unwrap();
/// let key = set.select(Some("key-a"), SigAlg::Rs256).unwrap();
/// assert_eq!(key.kid.as_deref(), Some("key-a"));
/// assert_eq!(key.modulus_bits, 2048);
/// assert_eq!(key.exponent, vec![0x01, 0x00, 0x01]); // 65537
/// assert_ne!(key.modulus[0], 0); // minimal big-endian
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsaPublicKey {
    /// The JWK `kid`, if the document published one.
    ///
    /// **Attacker-controlled.** See the security notes on
    /// [`JwkSet`](crate::jwks::keyset::JwkSet).
    pub kid: Option<String>,
    /// Big-endian RSA modulus `n`, with no leading zero bytes.
    pub modulus: Vec<u8>,
    /// Big-endian RSA public exponent `e`, with no leading zero bytes.
    pub exponent: Vec<u8>,
    /// Significant bits in `modulus`.
    pub modulus_bits: usize,
    /// The algorithm the JWK declared, if any.
    pub alg: Option<SigAlg>,
    /// The `key_ops` the JWK declared, if any. Always contains `verify` when set.
    pub key_ops: Option<Vec<String>>,
}
