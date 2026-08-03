//! Namespaced key derivation for the session store.
//!
//! # Why the prefix is validated too
//!
//! The prefix is normally a literal from the application, but it may be read from
//! configuration or an environment variable, so it goes through the same
//! [`super::sessionstore_validate::component`] check as the id. A prefix containing
//! `:` would create a *deeper* namespace than the caller wrote and could collide
//! with another application sharing the Redis instance.
//!
//! The id half is the security-critical one: it arrives in a cookie. See
//! [`super::sessionstore_validate`] for the key-injection argument.
//!
//! # Deliberately transport-free
//!
//! This returns a string. Nothing here opens a socket, so the Redis owner decides
//! command, TTL, and connection policy, and this logic stays unit-testable without
//! a server.

use super::sessionstore_validate::{component, SEP};

/// Derive the namespaced key for one session.
///
/// # Arguments
///
/// * `prefix` — Application namespace, e.g. `"sess"`.
/// * `session_id` — Untrusted session id, typically from a cookie.
///
/// # Returns
///
/// `"<prefix>:<session_id>"`.
///
/// # Errors
///
/// Returns a named error when either component is empty, contains `:`, or contains
/// a control character.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(derive("sess", "9f2c").unwrap(), "sess:9f2c");
/// assert!(derive("sess", "a:b").is_err());
/// ```
pub(super) fn derive(prefix: &str, session_id: &str) -> Result<String, String> {
    component("session_store_key: prefix", prefix)?;
    component("session_store_key: session_id", session_id)?;
    Ok(format!("{prefix}{SEP}{session_id}"))
}
