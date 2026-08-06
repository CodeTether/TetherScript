//! The public entry point: parse a JWKS document, then select a key from it.
//!
//! One responsibility: own the [`JwkSet`] type and its two operations. All policy
//! is delegated — bounds to [`crate::jwks::limits`], document shape to
//! [`crate::jwks::document`], per-key validation to `crate::jwks::parse_key`, and
//! choice to [`crate::jwks::select`].
//!
//! # Scope
//!
//! Parsing and selection **only**. This module performs no RSA arithmetic, no JWT
//! claim validation, and no HTTP fetching. It stops at a validated
//! [`RsaPublicKey`], which the caller hands to a verifier.
//!
//! # Security summary
//!
//! * `use: "enc"` keys and keys whose `key_ops` lacks `verify` are dropped at parse
//!   time, so they cannot reach a verifier by any path.
//! * A JWK `alg` that contradicts the requested algorithm is refused at selection.
//! * Only `kty: "RSA"` is implemented; every other `kty` — notably `oct`, which
//!   must never be read as RSA — is dropped rather than guessed at.
//! * `kid` is **attacker-controlled**: it selects among published keys and must
//!   never be used to build a filesystem path or a bare cache key. See
//!   [`crate::jwks::select`].
//! * With no `kid`, a tie among suitable keys is an error, never an arbitrary pick.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwks::alg::SigAlg;
//! use tetherscript::jwks::keyset::{EXAMPLE_JWKS, JwkSet};
//!
//! let set = JwkSet::parse(EXAMPLE_JWKS).unwrap();
//! let key = set.select(Some("key-a"), SigAlg::Rs256).unwrap();
//! assert_eq!(key.modulus_bits, 2048);
//! assert!(set.select(Some("key-b"), SigAlg::Rs256).is_err()); // an `enc` key
//! ```

use crate::jwks::alg::SigAlg;
use crate::jwks::document::parse_document;
use crate::jwks::error::JwksError;
use crate::jwks::key::RsaPublicKey;
use crate::jwks::skipped::SkippedKey;

pub use crate::jwks::example::EXAMPLE_JWKS;

/// The usable keys of a JWKS document, plus a record of what was dropped.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
///
/// let set = JwkSet::parse(r#"{"keys":[]}"#).unwrap();
/// assert!(set.keys().is_empty());
/// assert!(set.skipped().is_empty());
/// assert!(JwkSet::parse("{").is_err()); // malformed JSON
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwkSet {
    keys: Vec<RsaPublicKey>,
    skipped: Vec<SkippedKey>,
}

impl JwkSet {
    /// Parse a JWKS document body.
    ///
    /// # Arguments
    ///
    /// * `body` — The document source, as fetched from a JWKS endpoint.
    ///
    /// # Returns
    ///
    /// The set. An entry this module cannot use is dropped into
    /// [`skipped`](Self::skipped) rather than failing the document, because a realm
    /// legitimately publishes keys of several types.
    ///
    /// # Errors
    ///
    /// Returns a [`JwksError`] only for document-level faults: over the size or key
    /// bound, malformed JSON, a non-object top level, or a missing or non-array
    /// `keys` member.
    ///
    /// # Panics
    ///
    /// Does not panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::jwks::keyset::{EXAMPLE_JWKS, JwkSet};
    ///
    /// let set = JwkSet::parse(EXAMPLE_JWKS).unwrap();
    /// assert_eq!(set.keys().len(), 1);
    /// assert_eq!(set.skipped().len(), 1);
    /// ```
    pub fn parse(body: &str) -> Result<Self, JwksError> {
        let (keys, skipped) = parse_document(body)?;
        Ok(Self { keys, skipped })
    }

    /// The keys that passed validation, in document order.
    ///
    /// # Returns
    ///
    /// A slice of keys, each of which satisfies the contract documented on
    /// [`RsaPublicKey`].
    pub fn keys(&self) -> &[RsaPublicKey] {
        &self.keys
    }

    /// The entries that were dropped, with their reasons.
    ///
    /// # Returns
    ///
    /// A slice of skip records, in document order. Log these: they are the
    /// difference between "the realm publishes no RS256 key" and "the realm's
    /// RS256 key was refused for a 1024-bit modulus".
    pub fn skipped(&self) -> &[SkippedKey] {
        &self.skipped
    }

    /// Select the key to verify a token with.
    ///
    /// # Arguments
    ///
    /// * `kid` — The `kid` from the token's **unverified** header, if it carried
    ///   one. Attacker-controlled; see the module security summary.
    /// * `alg` — The algorithm the token claims.
    ///
    /// # Returns
    ///
    /// The single selected key.
    ///
    /// # Errors
    ///
    /// Returns [`JwksError::UnknownKid`], [`JwksError::UnsuitableKey`],
    /// [`JwksError::NoSuitableKey`], or [`JwksError::AmbiguousKey`]. With no `kid`,
    /// several suitable keys is [`JwksError::AmbiguousKey`] rather than an
    /// arbitrary choice.
    ///
    /// # Panics
    ///
    /// Does not panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::jwks::alg::SigAlg;
    /// use tetherscript::jwks::keyset::{EXAMPLE_JWKS, JwkSet};
    ///
    /// let set = JwkSet::parse(EXAMPLE_JWKS).unwrap();
    /// // Exactly one usable key, so a token with no `kid` still resolves.
    /// assert_eq!(set.select(None, SigAlg::Rs256).unwrap().kid.as_deref(), Some("key-a"));
    /// // A declared RS256 key cannot verify an RS512 token.
    /// assert!(set.select(Some("key-a"), SigAlg::Rs512).is_err());
    /// ```
    pub fn select(&self, kid: Option<&str>, alg: SigAlg) -> Result<&RsaPublicKey, JwksError> {
        crate::jwks::select::select(&self.keys, kid, alg)
    }
}
