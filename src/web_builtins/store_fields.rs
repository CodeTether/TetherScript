//! Field names and map assembly for the script-visible session shape.
//!
//! One place for the shape a script sees, so a rename cannot half-land across the
//! operation files.
//!
//! # What is deliberately absent
//!
//! Nothing here builds a cookie value. The cookie must carry the **id only**; see
//! the group docs on [`super::store`] for why.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::store_record::Record;
use crate::value::Value;

/// Key holding the session id.
pub(super) const ID: &str = "id";
/// Key holding the subject the session belongs to.
pub(super) const SUBJECT: &str = "subject";
/// Key holding the application payload map.
pub(super) const DATA: &str = "data";
/// Key holding creation time in Unix milliseconds.
pub(super) const CREATED_MS: &str = "created_ms";
/// Key holding last-activity time in Unix milliseconds.
pub(super) const SEEN_MS: &str = "seen_ms";
/// Key holding the idle window in milliseconds.
pub(super) const IDLE_TTL_MS: &str = "idle_ttl_ms";
/// Key holding the absolute ceiling in milliseconds.
pub(super) const ABSOLUTE_TTL_MS: &str = "absolute_ttl_ms";

/// Render a stored record as the map a script receives.
///
/// # Arguments
///
/// * `record` — The stored session.
///
/// # Returns
///
/// A fresh map. `data` is a fresh map too, so mutating what a script got back
/// cannot reach into the store without an explicit `store_save`.
pub(super) fn session_map(record: &Record) -> Value {
    let mut out = HashMap::new();
    out.insert(ID.into(), Value::Str(Rc::new(record.id.clone())));
    out.insert(SUBJECT.into(), Value::Str(Rc::new(record.subject.clone())));
    let data = Value::Map(Rc::new(RefCell::new(record.data.clone())));
    out.insert(DATA.into(), data);
    out.insert(CREATED_MS.into(), Value::Int(record.created_ms));
    out.insert(SEEN_MS.into(), Value::Int(record.seen_ms));
    out.insert(IDLE_TTL_MS.into(), Value::Int(record.idle_ttl_ms));
    out.insert(ABSOLUTE_TTL_MS.into(), Value::Int(record.absolute_ttl_ms));
    Value::Map(Rc::new(RefCell::new(out)))
}
