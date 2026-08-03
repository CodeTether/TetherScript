//! Constant-time session id comparison.
//!
//! # Which comparisons need it, and which do not
//!
//! **Needs it:** comparing a *client-presented* id against a *known-good* one.
//! That comparison is an oracle. A byte-wise `==` short-circuits at the first
//! mismatch, so an attacker able to measure many attempts learns the correct prefix
//! one byte at a time and reduces a 256-bit guess to a few hundred queries. Every
//! confirmation step in this group routes through [`ids_match`].
//!
//! **Does not need it:** the hash-map probe that locates the record. A map lookup's
//! timing depends on the key by construction and no comparison discipline can hide
//! that; what protects the lookup is the entropy of the id, not the compare. Also
//! exempt: comparing a *subject*, which is a user identifier rather than a secret
//! and arrives from the already-authenticated server side; and comparing two ids
//! the server itself generated, as rotation does, where the client supplied neither
//! side and so learns nothing from the timing.
//!
//! Claiming the map probe is constant-time would be false, so it is stated plainly
//! rather than papered over.

/// Compare a presented id against a stored one without leaking where they differ.
///
/// Delegates to [`super::super::hmac::constant_time_eq`] rather than carrying a
/// second implementation, so the tree holds one audited byte-folding loop.
///
/// # Arguments
///
/// * `presented` — Id supplied by the client.
/// * `stored` — Id held by the store.
///
/// # Returns
///
/// True when the two are byte-identical. Differing lengths return false at once:
/// an id's length is fixed and public, its contents are not.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(ids_match("abc", "abc"));
/// assert!(!ids_match("abc", "abd"));
/// ```
pub(super) fn ids_match(presented: &str, stored: &str) -> bool {
    super::super::hmac::constant_time_eq(presented.as_bytes(), stored.as_bytes())
}
