//! Session id rotation, and why it is the fix for session fixation.
//!
//! # Session fixation
//!
//! In a fixation attack the attacker obtains or plants a session id *before* the
//! victim authenticates — by setting a cookie from a sibling subdomain, by an
//! injected `Set-Cookie`, or by handing the victim a link that seeds one. The victim
//! then logs in. If the server keeps the same id across that privilege change, the
//! attacker's pre-known id is now an authenticated session, and no credential was
//! ever stolen.
//!
//! The fix is not entropy — the id was already unguessable, the attacker simply knew
//! it. The fix is to **issue a new id at every privilege change** and drop the old
//! one, so the id the attacker holds names nothing. Rotate on login, on step-up
//! authentication such as MFA, on assuming elevated privileges, and after a password
//! change. OWASP states this as a requirement, not a hardening tip.
//!
//! Rotation is only complete once the old key is deleted, which is the Redis owner's
//! step; this half mints the replacement and proves it differs.

use super::sessionstore_id::{generate, ids_match};
use super::sessionstore_validate::component;

/// Mint a replacement for an existing session id.
///
/// # Arguments
///
/// * `old_id` — The id being retired. Validated because it typically arrives from a
///   cookie, so an unusable value should be reported at rotation rather than turned
///   into a key later.
///
/// # Returns
///
/// A fresh id, guaranteed different from `old_id`.
///
/// # Errors
///
/// Returns a named error when `old_id` is empty, contains the key separator, or
/// contains a control character.
///
/// # Examples
///
/// ```rust,ignore
/// let next = rotate("9f2c").unwrap();
/// assert_ne!(next, "9f2c");
/// ```
pub(super) fn rotate(old_id: &str) -> Result<String, String> {
    component("session_rotate_id: old_id", old_id)?;
    let mut fresh = generate();
    // With 256 bits a collision will not occur; the loop is a correctness guarantee
    // rather than an expected path, because returning the old id would silently make
    // rotation a no-op and leave the fixation window open. Compared in constant time
    // because the argument may be attacker-supplied.
    while ids_match(&fresh, old_id) {
        fresh = generate();
    }
    Ok(fresh)
}
