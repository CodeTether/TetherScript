//! A record of one JWK that was dropped, and why.
//!
//! One responsibility: carry the diagnostic for a skipped key.
//!
//! # Why skips are values rather than errors
//!
//! A realm legitimately publishes keys this module cannot use: EC keys, `oct`
//! entries, encryption keys, keys whose `alg` it does not implement. Failing the
//! whole document on the first such entry would make a correct JWKS unusable —
//! so an unusable key is dropped and recorded. Keeping the reason means the
//! difference between "the realm has no RS256 key" and "the realm's RS256 key was
//! rejected for a 1024-bit modulus" is visible in a log instead of being guessed
//! at during an incident.

/// One JWK that was not usable, with its position and reason.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::keyset::JwkSet;
///
/// // An `oct` entry alongside nothing else: the document parses, the key is
/// // dropped, and the reason names the `kty` that was refused.
/// let set = JwkSet::parse(r#"{"keys":[{"kty":"oct","kid":"h","k":"AAAA"}]}"#).unwrap();
/// assert_eq!(set.keys().len(), 0);
/// assert_eq!(set.skipped().len(), 1);
/// assert_eq!(set.skipped()[0].index, 0);
/// assert!(set.skipped()[0].reason.contains("oct"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedKey {
    /// Zero-based position of the entry in the `keys` array.
    pub index: usize,
    /// The JWK `kid`, when it could be read. `None` when absent or unreadable.
    pub kid: Option<String>,
    /// Why the key was dropped. Names the member or rule that refused it.
    pub reason: String,
}
