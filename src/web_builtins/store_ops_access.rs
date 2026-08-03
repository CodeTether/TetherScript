//! `store_load`, `store_save`, and `store_touch`: operating on a live session.
//!
//! All three reject an unknown, destroyed, or expired id, because all three take a
//! client-presented id and none may act on a credential the server no longer
//! honours.

use super::store_args::{map_arg, str_arg};
use super::store_clock::now_ms;
use super::store_fields::session_map;
use super::store_lookup::live;
use super::store_state;
use super::store_write;
use crate::value::Value;

/// `store_load(id)` — fetch a live session.
///
/// # Arguments
///
/// * `args` — `[id: str]`.
///
/// # Returns
///
/// The session map. `data` is a copy, so editing it does not reach the store until
/// `store_save`.
///
/// # Errors
///
/// Returns `store_load: no session with that id` for an unknown *or destroyed* id —
/// the two are deliberately indistinguishable, since reporting "revoked" confirms
/// the id once existed — and a named expiry error when a clock has lapsed.
pub(super) fn load(args: &[Value]) -> Result<Value, String> {
    let id = str_arg(&args[0], "store_load: id")?;
    let now = now_ms();
    store_state::with(|store| {
        let record = live(&*store.backend, "store_load", &id, now)?;
        Ok(session_map(&record))
    })
}

/// `store_save(id, data)` — replace the payload wholesale.
///
/// # Arguments
///
/// * `args` — `[id: str, data: map | nil]`.
///
/// # Returns
///
/// The saved session map, with `seen_ms` advanced.
///
/// # Errors
///
/// Returns a named error for a bad argument type, an unknown id, or an expired
/// session.
pub(super) fn save(args: &[Value]) -> Result<Value, String> {
    let id = str_arg(&args[0], "store_save: id")?;
    let data = map_arg(&args[1], "store_save: data")?;
    let now = now_ms();
    store_state::with(|store| {
        let saved = store_write::save(&mut *store.backend, "store_save", &id, data, now)?;
        Ok(session_map(&saved))
    })
}

/// `store_touch(id)` — extend the idle window only.
///
/// # Arguments
///
/// * `args` — `[id: str]`.
///
/// # Returns
///
/// The session map with a fresh `seen_ms` and an unchanged `created_ms`, so the
/// absolute ceiling still falls where it always did.
///
/// # Errors
///
/// Returns a named error for a bad argument type, an unknown id, or an expired
/// session.
pub(super) fn touch(args: &[Value]) -> Result<Value, String> {
    let id = str_arg(&args[0], "store_touch: id")?;
    let now = now_ms();
    store_state::with(|store| {
        let touched = store_write::touch(&mut *store.backend, "store_touch", &id, now)?;
        Ok(session_map(&touched))
    })
}
