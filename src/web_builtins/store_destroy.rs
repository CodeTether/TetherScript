//! Revocation: destroying sessions server-side.
//!
//! # Why this is the whole point of a server-side store
//!
//! A signed cookie is self-contained: the server hands one out and then has no say
//! in it until the `exp` inside it passes. There is nothing to revoke. A stolen
//! cookie stays valid for its full TTL, and "log out everywhere", "ban this
//! account", and "invalidate every session after a password change" are all
//! unimplementable. `docs/web-builtins.md` records the related property that
//! `session_sign` produces a signed but *not* encrypted value, so the payload is
//! not merely unrevocable but readable too.
//!
//! With a store, the cookie is only a *pointer*. Deleting the record makes a
//! perfectly-signed, perfectly-unexpired cookie resolve to nothing, so it is useless
//! the instant the delete lands. That is the capability the signed-cookie half
//! structurally cannot provide.

use super::store_backend::SessionBackend;

/// Destroy one session.
///
/// # Arguments
///
/// * `backend` — Storage to delete from.
/// * `id` — Session id to revoke.
///
/// # Returns
///
/// True when a record was present and is now gone; false when the id was already
/// unknown. Deliberately not an error: logout must be idempotent, so a double submit
/// or a retried request cannot produce a spurious failure.
///
/// # Errors
///
/// Propagates a transport failure. A store that could not confirm the delete must
/// not report success, or a "logged out everywhere" screen would be a lie.
pub(super) fn destroy(backend: &mut dyn SessionBackend, id: &str) -> Result<bool, String> {
    backend.delete(id)
}

/// Destroy every session belonging to one subject.
///
/// The "log out everywhere" and post-password-change sweep. It matches `subject`
/// with an ordinary `==`: a subject is a user identifier rather than a secret and
/// arrives from the already-authenticated server side, so there is no guess for a
/// timing signal to confirm.
///
/// # Arguments
///
/// * `backend` — Storage to delete from.
/// * `subject` — Whose sessions to remove.
///
/// # Returns
///
/// How many sessions were removed. `0` is a normal answer.
///
/// # Errors
///
/// Propagates a transport failure.
pub(super) fn destroy_subject(
    backend: &mut dyn SessionBackend,
    subject: &str,
) -> Result<usize, String> {
    backend.delete_subject(subject)
}
