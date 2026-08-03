//! Fetching a live record, applying both expiry rules on the way out.
//!
//! Every read path goes through here, so no operation can accidentally act on an
//! expired session. Expiry is enforced on *read* rather than by a background
//! reaper because the runtime is a cooperative single-threaded scheduler with no
//! timer thread to run one; `store_sweep` reclaims memory separately.

use super::store_backend::SessionBackend;
use super::store_compare::ids_match;
use super::store_expiry;
use super::store_record::Record;

/// Load a record by id and reject it if either clock has lapsed.
///
/// # Arguments
///
/// * `backend` — Storage to read from.
/// * `label` — Built-in name, so the failure says which call rejected the id.
/// * `id` — Client-presented session id.
/// * `now_ms` — Current Unix milliseconds.
///
/// # Returns
///
/// The live record.
///
/// # Errors
///
/// Returns `"<label>: no session with that id"` for an unknown or destroyed id —
/// the two are deliberately indistinguishable, since telling a caller an id was
/// *revoked* confirms it once existed. Returns the expiry message from
/// [`store_expiry::message`] when a clock has lapsed. Propagates a transport
/// failure unchanged.
pub(super) fn live(
    backend: &dyn SessionBackend,
    label: &str,
    id: &str,
    now_ms: i64,
) -> Result<Record, String> {
    let Some(record) = backend.load(id)? else {
        return Err(format!("{label}: no session with that id"));
    };
    // Constant-time confirmation of the presented id against the stored one. The
    // map probe above is not constant-time and cannot be; see `store_compare`.
    if !ids_match(id, &record.id) {
        return Err(format!("{label}: no session with that id"));
    }
    match store_expiry::evaluate(&record, now_ms) {
        Some(reason) => Err(store_expiry::message(label, &reason)),
        None => Ok(record),
    }
}
