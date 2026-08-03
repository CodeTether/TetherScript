//! Session id generation, rotation, and secret-safe comparison.
//!
//! # Entropy
//!
//! An id is [`ID_BYTES`] = 32 bytes of OS entropy rendered as lowercase hex: 64
//! characters carrying **256 bits**.
//!
//! Why 256. A session id is a bearer credential — presenting it *is* the
//! authentication — so there is no password, prompt, or lockout an attacker must
//! first defeat, and the size of the search space is the entire defence. OWASP's
//! floor for a session identifier is 64 bits and 128 is the common modern figure.
//! At 256 bits an attacker testing 2^40 ids per second for 2^35 seconds still
//! covers about 2^-181 of the space. The extra 32 characters cost nothing against a
//! ~4096-byte cookie budget, so there is no reason to economise.
//!
//! Hex, not base64url, because hex never contains [`super::sessionstore_validate::SEP`],
//! so a freshly minted id can never be rejected by its own key validator and no
//! backend has to re-encode it.

use super::sessionstore_entropy::bytes;
use crate::system::hex_encode;

/// Session id width in bytes, before hex encoding.
pub(super) const ID_BYTES: usize = 32;

/// Mint a fresh session id.
///
/// # Returns
///
/// 64 lowercase hex characters carrying 256 bits of entropy.
///
/// # Errors
///
/// Infallible; see [`super::sessionstore_entropy`] for the degraded-entropy path.
///
/// # Examples
///
/// ```rust,ignore
/// let id = generate();
/// assert_eq!(id.len(), 64);
/// assert_ne!(id, generate());
/// ```
pub(super) fn generate() -> String {
    hex_encode(&bytes(ID_BYTES))
}

/// Compare a presented id against a stored one without leaking where they differ.
///
/// Delegates to the tree's single audited byte-folding loop rather than carrying a
/// second copy. A plain `==` short-circuits at the first mismatching byte, which
/// turns a 256-bit guess into a few hundred timed queries.
///
/// # Arguments
///
/// * `presented` — Id supplied by the client.
/// * `stored` — Id held by the server.
///
/// # Returns
///
/// True when the two are byte-identical. Differing lengths return false at once: an
/// id's length is fixed and public, its contents are not.
pub(super) fn ids_match(presented: &str, stored: &str) -> bool {
    super::super::hmac::constant_time_eq(presented.as_bytes(), stored.as_bytes())
}
