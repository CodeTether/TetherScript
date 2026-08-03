//! `store_destroy`, `store_destroy_subject`, and `store_sweep`.
//!
//! Revocation is separated from the access operations because it is the reason the
//! store exists at all; see [`super::store_destroy`] for the argument.

use super::store_args::str_arg;
// The trait must be in scope to call `sweep` through the boxed backend.
// The concrete backend types are reached directly; the trait is not named here.
use super::store_clock::now_ms;
use super::store_destroy;
use super::store_state;
use crate::value::Value;

/// `store_destroy(id)` — revoke one session immediately.
///
/// # Arguments
///
/// * `args` — `[id: str]`.
///
/// # Returns
///
/// True when a session was removed, false when the id was already unknown. Logout
/// is idempotent, so an unknown id is not an error.
///
/// # Errors
///
/// Returns a named error when `id` is not a str, or when the backend fails.
pub(super) fn destroy(args: &[Value]) -> Result<Value, String> {
    let id = str_arg(&args[0], "store_destroy: id")?;
    store_state::with(|store| {
        let gone = store_destroy::destroy(&mut *store.backend, &id)?;
        Ok(Value::Bool(gone))
    })
}

/// `store_destroy_subject(subject)` — log a subject out everywhere.
///
/// # Arguments
///
/// * `args` — `[subject: str]`.
///
/// # Returns
///
/// The number of sessions removed; `0` when the subject had none.
///
/// # Errors
///
/// Returns a named error when `subject` is not a str, or when the backend fails.
pub(super) fn destroy_subject(args: &[Value]) -> Result<Value, String> {
    let subject = str_arg(&args[0], "store_destroy_subject: subject")?;
    store_state::with(|store| {
        let removed = store_destroy::destroy_subject(&mut *store.backend, &subject)?;
        Ok(Value::Int(removed as i64))
    })
}

/// `store_sweep()` — drop expired records to reclaim space.
///
/// # Returns
///
/// The number of records dropped. This changes no answer a script can observe,
/// because expiry is already enforced on every read; it only frees memory.
///
/// # Errors
///
/// Returns an error only when the backend fails.
pub(super) fn sweep() -> Result<Value, String> {
    let now = now_ms();
    store_state::with(|store| {
        let dropped = store.backend.sweep(now)?;
        Ok(Value::Int(dropped as i64))
    })
}
