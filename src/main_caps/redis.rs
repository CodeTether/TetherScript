//! Redis capability selection for CLI grants.
//!
//! One concern: turning a `--grant-redis redis://…` argument into a
//! [`RedisAuthority`]. The URL parsing itself lives in
//! [`redis_cap::url`](crate::redis_cap::url) and is reached here through
//! [`super::redis_url`], so it is also reachable from the integration tests; this file
//! only decides *whether* to build an authority, and connects.
//!
//! # No grant means no capability
//!
//! `Ok(None)` leaves the global name `redis` unbound, so a script that references it
//! gets an undefined-variable error rather than an ambient connection to
//! `127.0.0.1:6379`. Mirrors `main_caps::db`.
//!
//! Like `--grant-db` and unlike `--grant-fs`, this is **never** implied by
//! `--access-mode full`: a Redis URL carries credentials and a database index that
//! cannot be guessed from the environment, so the grant must always be explicit.

use crate::redis_cap::RedisAuthority;

/// Build the `redis` authority for an explicit `--grant-redis` argument.
///
/// # Arguments
///
/// * `explicit` — The URL, or `None` when the flag was absent.
///
/// # Returns
///
/// `Ok(None)` when no grant was requested, so the `redis` name stays undefined.
///
/// # Errors
///
/// Returns an error when the URL cannot be parsed or the connection, `AUTH`, or
/// `SELECT` fails. Failing here rather than lazily means a bad password surfaces while
/// the CLI is still reading its flags, not midway through a script. The message never
/// contains the URL or the password.
pub(super) fn authority(explicit: &Option<String>) -> Result<Option<RedisAuthority>, String> {
    let Some(target) = explicit else {
        return Ok(None);
    };
    let config = super::redis_url::parse_url(target)?;
    RedisAuthority::connect(&config)
        .map(Some)
        .map_err(|error| format!("--grant-redis: {error}"))
}
