//! The RSA signature algorithms this module is willing to select a key for.
//!
//! One responsibility: turn an algorithm name into a closed enum, and compare it
//! against a JWK's declared `alg`.
//!
//! # Security: a closed set, checked before any key is looked at
//!
//! Parsing the requested algorithm first means an `HS256` or `none` request is
//! refused before selection begins, so a symmetric or unsigned algorithm can
//! never be paired with a public key. That pairing is the *algorithm confusion*
//! attack: the attacker rewrites a token header to `HS256`, and a verifier that
//! looks up the RSA key by `kid` and then HMACs with the modulus as the secret
//! accepts a token they forged from public data.

use crate::jwks::error::JwksError;

/// An RSA-family JWS signature algorithm (RFC 7518 §3.3).
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::alg::SigAlg;
///
/// assert_eq!(SigAlg::parse("RS256").unwrap(), SigAlg::Rs256);
/// assert_eq!(SigAlg::Rs512.name(), "RS512");
/// // Symmetric and unsigned algorithms are refused, not mapped.
/// assert!(SigAlg::parse("HS256").is_err());
/// assert!(SigAlg::parse("none").is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigAlg {
    /// RSASSA-PKCS1-v1_5 using SHA-256.
    Rs256,
    /// RSASSA-PKCS1-v1_5 using SHA-384.
    Rs384,
    /// RSASSA-PKCS1-v1_5 using SHA-512.
    Rs512,
}

impl SigAlg {
    /// Parse a JWS `alg` name.
    ///
    /// # Arguments
    ///
    /// * `name` — The algorithm name, compared case-sensitively as registered.
    ///
    /// # Returns
    ///
    /// The matching variant.
    ///
    /// # Errors
    ///
    /// Returns [`JwksError::UnsupportedAlgorithm`] for anything outside the
    /// RSA-PKCS1 family, including `HS*`, `ES*`, `PS*`, and `none`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn parse(name: &str) -> Result<Self, JwksError> {
        match name {
            "RS256" => Ok(Self::Rs256),
            "RS384" => Ok(Self::Rs384),
            "RS512" => Ok(Self::Rs512),
            other => Err(JwksError::UnsupportedAlgorithm(other.to_string())),
        }
    }

    /// The registered name of this algorithm.
    ///
    /// # Returns
    ///
    /// A static string such as `"RS256"`, suitable for error text and for
    /// comparison against a JWK's `alg` member.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rs256 => "RS256",
            Self::Rs384 => "RS384",
            Self::Rs512 => "RS512",
        }
    }
}
