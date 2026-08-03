//! `store_create` and `store_rotate`: the two operations that mint an id.
//!
//! Grouped because both produce a *new* id, and the invariant they share — the
//! caller must send the returned id to the client, or the new session is
//! unreachable — is easiest to keep right when they sit side by side.

use super::store_args::{map_arg, str_arg};
use super::store_clock::now_ms;
use super::store_create;
use super::store_fields::session_map;
use super::store_state;
use crate::value::Value;

/// `store_create(subject, data)` — start a session and return it.
///
/// # Arguments
///
/// * `args` — `[subject: str, data: map | nil]`.
///
/// # Returns
///
/// The session map, including the generated `id`. Send **only** that id to the
/// client; see the group docs on why the payload must stay server-side.
///
/// # Errors
///
/// Returns a named error when `subject` is not a str or `data` is neither map nor
/// nil, or when the backend rejects the insert.
pub(super) fn create(args: &[Value]) -> Result<Value, String> {
    let subject = str_arg(&args[0], "store_create: subject")?;
    let data = map_arg(&args[1], "store_create: data")?;
    let now = now_ms();
    store_state::with(|store| {
        let ttls = (store.idle_ttl_ms, store.absolute_ttl_ms);
        let record = store_create::create(&mut *store.backend, subject, data, ttls, now)?;
        Ok(session_map(&record))
    })
}

/// `store_rotate(id)` — replace the id, keeping the data. Call this on login.
///
/// # Arguments
///
/// * `args` — `[id: str]`, the session's current id.
///
/// # Returns
///
/// The session map under its new `id`. The old id no longer resolves.
///
/// # Errors
///
/// Returns a named error when `id` is not a str, is unknown, or has expired.
pub(super) fn rotate(args: &[Value]) -> Result<Value, String> {
    let id = str_arg(&args[0], "store_rotate: id")?;
    let now = now_ms();
    store_state::with(|store| {
        let record = store_create::rotate(&mut *store.backend, "store_rotate", &id, now)?;
        Ok(session_map(&record))
    })
}
