//! # Digest algorithm identifiers for PKCS#1 v1.5
//!
//! One responsibility: name the hash functions this verifier accepts and expose
//! the two facts EMSA-PKCS1-v1_5 needs about each one — its DER `DigestInfo`
//! prefix and its digest length in octets. The prefix bytes themselves live in
//! `super::digestinfo_prefix` and are quoted from RFC 8017 section 9.2.
//!
//! This type deliberately does **not** compute digests. The caller hashes the
//! signing input with its own SHA implementation and hands the digest to
//! [`verify`](fn@crate::rsa::verify); keeping hashing out of this module means the
//! verifier has
//! exactly one job.
//!
//! ## Integration
//!
//! The integrator wires this with `mod digestinfo;` plus
//! `pub use digestinfo::DigestAlgorithm;` in the `rsa` module root, which is the
//! path the examples below use.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::DigestAlgorithm;
//!
//! let alg = DigestAlgorithm::Sha256;
//! assert_eq!(alg.digest_len(), 32);
//! // 19 prefix octets + 32 digest octets = 51-octet DigestInfo.
//! assert_eq!(alg.encoded_len(), 51);
//! ```

use super::digestinfo_prefix as prefix;

/// A hash function usable with RSASSA-PKCS1-v1_5 verification.
///
/// # Examples
///
/// ```rust
/// use tetherscript::rsa::DigestAlgorithm;
///
/// for alg in [
///     DigestAlgorithm::Sha1,
///     DigestAlgorithm::Sha256,
///     DigestAlgorithm::Sha384,
///     DigestAlgorithm::Sha512,
/// ] {
///     // The final DigestInfo prefix octet is the OCTET STRING length, which
///     // must agree with the digest length. RFC 8017 section 9.2 note 1.
///     let tail = *alg.der_prefix().last().unwrap() as usize;
///     assert_eq!(tail, alg.digest_len());
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DigestAlgorithm {
    /// SHA-1. Accepted for legacy interoperability only; it is collision-broken
    /// and must not be chosen for new signatures.
    Sha1,
    /// SHA-256, the hash behind the JWS `RS256` algorithm.
    Sha256,
    /// SHA-384, the hash behind the JWS `RS384` algorithm.
    Sha384,
    /// SHA-512, the hash behind the JWS `RS512` algorithm.
    Sha512,
}

impl DigestAlgorithm {
    /// The exact DER `DigestInfo` prefix that precedes the digest.
    ///
    /// # Returns
    ///
    /// A `'static` slice of octets from RFC 8017 section 9.2. Never empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::rsa::DigestAlgorithm;
    ///
    /// assert_eq!(DigestAlgorithm::Sha1.der_prefix().len(), 15);
    /// ```
    pub fn der_prefix(self) -> &'static [u8] {
        match self {
            Self::Sha1 => prefix::SHA1,
            Self::Sha256 => prefix::SHA256,
            Self::Sha384 => prefix::SHA384,
            Self::Sha512 => prefix::SHA512,
        }
    }

    /// Length in octets of a digest produced by this algorithm.
    ///
    /// # Returns
    ///
    /// 20, 32, 48, or 64.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::rsa::DigestAlgorithm;
    ///
    /// assert_eq!(DigestAlgorithm::Sha512.digest_len(), 64);
    /// ```
    pub fn digest_len(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    /// Total octets occupied by the DER `DigestInfo` for this algorithm.
    ///
    /// # Returns
    ///
    /// `der_prefix().len() + digest_len()`, i.e. the `tLen` of RFC 8017
    /// section 9.2 step 2.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::rsa::DigestAlgorithm;
    ///
    /// assert_eq!(DigestAlgorithm::Sha384.encoded_len(), 19 + 48);
    /// ```
    pub fn encoded_len(self) -> usize {
        self.der_prefix().len() + self.digest_len()
    }
}
