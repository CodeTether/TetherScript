//! Writes against a live session: extending the idle clock, and replacing data.
//!
//! # Why neither can extend the absolute lifetime
//!
//! Both move `seen_ms` forward and leave `created_ms` alone. That asymmetry *is* the
//! security property: if activity extended both clocks, an attacker holding a stolen
//! id would keep the session alive indefinitely simply by using it and the ceiling
//! would never bind. The reference application's extend-on-every-request TTL has
//! exactly that weakness; the ceiling here is what fixes it.
//!
//! # Why save replaces rather than merges
//!
//! A merge cannot express *deletion*, so clearing a role or a pending-MFA flag would
//! be impossible and a stale elevated claim would survive a downgrade. Callers who
//! want a merge read `session.data`, edit it, and save the result.

use std::collections::HashMap;

use super::store_backend::SessionBackend;
use super::store_lookup::live;
use super::store_record::Record;
use crate::value::Value;

/// Move a session's last-activity time to now.
///
/// # Arguments
///
/// * `backend` — Storage holding the session.
/// * `label` — Built-in name, for error wording.
/// * `id` — Client-presented session id.
/// * `now_ms` — Current Unix milliseconds.
///
/// # Returns
///
/// The saved record with a refreshed `seen_ms` and an unchanged `created_ms`, so the
/// absolute ceiling still falls at its original time.
///
/// # Errors
///
/// Returns the unknown-id or expired error from [`live`]: touching an already
/// expired session must fail rather than resurrect a credential the server has
/// already stopped honouring.
pub(super) fn touch(
    backend: &mut dyn SessionBackend,
    label: &str,
    id: &str,
    now_ms: i64,
) -> Result<Record, String> {
    let mut record = live(backend, label, id, now_ms)?;
    record.seen_ms = now_ms;
    backend.save(record.clone())?;
    Ok(record)
}

/// Replace a session's data and refresh its idle clock.
///
/// # Arguments
///
/// * `backend` — Storage holding the session.
/// * `label` — Built-in name, for error wording.
/// * `id` — Client-presented session id.
/// * `data` — New payload, replacing the old one entirely.
/// * `now_ms` — Current Unix milliseconds.
///
/// # Returns
///
/// The saved record. `seen_ms` advances, because a write is activity.
///
/// # Errors
///
/// Returns the unknown-id or expired error from [`live`].
pub(super) fn save(
    backend: &mut dyn SessionBackend,
    label: &str,
    id: &str,
    data: HashMap<String, Value>,
    now_ms: i64,
) -> Result<Record, String> {
    let mut record = live(backend, label, id, now_ms)?;
    record.data = data;
    record.seen_ms = now_ms;
    backend.save(record.clone())?;
    Ok(record)
}
