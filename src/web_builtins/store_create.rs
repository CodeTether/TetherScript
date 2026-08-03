//! Creating a session and rotating its id — the session-fixation defence.
//!
//! # The attack
//!
//! 1. The attacker obtains a valid session id from the site, trivially by visiting
//!    it, or plants one of their choosing in the victim's browser via a URL
//!    parameter, a `Set-Cookie` from a sibling subdomain, or an XSS write.
//! 2. The victim arrives already carrying **that** id and logs in.
//! 3. If the server keeps the same id and merely marks the existing session
//!    authenticated, the attacker's copy is now an authenticated session. No
//!    credential was ever stolen; the attacker supplied the id.
//!
//! The fix is to mint a **new** id at every privilege change — login, elevation,
//! impersonation — and drop the old one, leaving the attacker's id attached to a
//! destroyed session. Preserving the data across the rotation is the other half of
//! the point: a pre-login cart or CSRF token must survive, or applications skip
//! rotation to avoid losing it.
//!
//! # Order matters in [`rotate`]
//!
//! The new record is inserted *before* the old id is deleted. If the insert fails,
//! the caller still holds a working session and can retry; deleting first would log
//! the user out mid-login on a transient backend error. The window in which both
//! ids resolve is one statement wide, single-threaded, and closed before return.
//!
//! `created_ms` restarts on rotation: a new authenticated session begins here and
//! earns a full absolute lifetime. Carrying the anonymous session's age forward
//! would fire the ceiling mid-session for a user who browsed a long time before
//! logging in, which again pushes applications to skip rotation.

use std::collections::HashMap;

use super::store_backend::SessionBackend;
use super::store_id::generate;
use super::store_lookup::live;
use super::store_record::Record;
use crate::value::Value;

/// Build and insert a fresh session for `subject`.
///
/// # Arguments
///
/// * `backend` — Storage to insert into.
/// * `subject` — Whose session this is.
/// * `data` — Initial payload; may be empty.
/// * `ttls` — `(idle_ttl_ms, absolute_ttl_ms)`, copied onto the record so a later
///   policy change cannot retroactively lengthen a live session.
/// * `now_ms` — Current Unix milliseconds; becomes both `created_ms` and `seen_ms`.
///
/// # Returns
///
/// The stored record, including its freshly generated id.
///
/// # Errors
///
/// Propagates a backend failure, including the id-collision error.
pub(super) fn create(
    backend: &mut dyn SessionBackend,
    subject: String,
    data: HashMap<String, Value>,
    ttls: (i64, i64),
    now_ms: i64,
) -> Result<Record, String> {
    let record = Record {
        id: generate(),
        subject,
        data,
        created_ms: now_ms,
        seen_ms: now_ms,
        idle_ttl_ms: ttls.0,
        absolute_ttl_ms: ttls.1,
    };
    backend.create(record.clone())?;
    Ok(record)
}

/// Replace a session's id, preserving its data.
///
/// # Arguments
///
/// * `backend` — Storage holding the session.
/// * `label` — Built-in name, for error wording.
/// * `id` — Current, client-presented session id.
/// * `now_ms` — Current Unix milliseconds.
///
/// # Returns
///
/// The new record: a different `id`, the same `subject` and `data`, both clocks
/// restarted.
///
/// # Errors
///
/// Returns the unknown-id or expired error from [`live`]. Rotating an expired
/// session would launder a dead credential into a live one.
pub(super) fn rotate(
    backend: &mut dyn SessionBackend,
    label: &str,
    id: &str,
    now_ms: i64,
) -> Result<Record, String> {
    let old = live(backend, label, id, now_ms)?;
    let ttls = (old.idle_ttl_ms, old.absolute_ttl_ms);
    let fresh = create(backend, old.subject.clone(), old.data.clone(), ttls, now_ms)?;
    backend.delete(&old.id)?;
    Ok(fresh)
}
