//! Session id generation.
//!
//! Comparison lives in [`super::store_compare`], because generating a secret and
//! comparing one are different concerns with different failure modes.
//!
//! # Entropy
//!
//! An id is [`ID_BYTES`] = 32 bytes drawn from the OS CSPRNG and rendered as
//! lowercase hex: 64 characters carrying **256 bits** of entropy.
//!
//! Why 256 bits is sufficient. A session id is a bearer credential — presenting it
//! *is* the authentication — so there is no password prompt or lockout an attacker
//! must first defeat and the search space is the entire defence. OWASP's floor for
//! a session identifier is 64 bits and 128 is the common modern figure. At 256
//! bits an attacker testing 2^40 ids per second (about a trillion) for 2^35 seconds
//! (longer than the age of the universe) would still have covered roughly 2^-181 of
//! the space. Choosing 256 over 128 costs 32 extra characters against a cookie
//! budget of about 4096 bytes, so there is nothing to gain by economising.
//!
//! # Reuse, not reinvention
//!
//! The bytes come from [`super::store_entropy`], which performs the same fixed-size
//! `/dev/urandom` read, with the same documented time-and-PID fallback, as the
//! `random` group's `random_source.rs`. That module is `mod`-private to its own
//! group and this group may not edit it; the file explains the one-line change that
//! collapses the two into one.

use super::store_entropy::bytes;
use crate::system::hex_encode;

/// Session id width in bytes, before hex encoding.
pub(super) const ID_BYTES: usize = 32;

/// Mint a fresh session id.
///
/// # Returns
///
/// 64 lowercase hex characters carrying 256 bits of entropy. Hex needs no escaping
/// in a cookie value, a URL path segment, or a Redis key, so no backend has to
/// re-encode it, and it cannot trip `cookie_serialize`'s injection guard.
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
