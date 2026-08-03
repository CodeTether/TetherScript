//! Integer parsing for the two stream bounds.
//!
//! Split from [`super`] so the `Bounds` type and its per-key validation stay in
//! separate files. Clamping to a ceiling rather than erroring is deliberate: a
//! handler asking for a longer stream than the server will tolerate should still
//! work, just less than it hoped, because refusing the whole response would take
//! down a route over a tuning value.

use std::collections::HashMap;

use crate::value::Value;

use super::{DEFAULT_MAX_DURATION_MS, DEFAULT_MAX_EVENTS, DURATION_CEILING_MS, EVENT_CEILING};

/// Read `max_events`, defaulting and clamping as documented on [`super::Bounds`].
///
/// # Arguments
///
/// * `map` — Borrowed streaming-response map.
///
/// # Returns
///
/// The event cap, at most [`EVENT_CEILING`].
///
/// # Errors
///
/// Returns `Err` naming `max_events` when the value is not a positive int.
pub(super) fn events(map: &HashMap<String, Value>) -> Result<u32, String> {
    match map.get("max_events") {
        None | Some(Value::Nil) => Ok(DEFAULT_MAX_EVENTS),
        Some(Value::Int(count)) if *count > 0 => {
            Ok((*count as u64).min(EVENT_CEILING as u64) as u32)
        }
        Some(Value::Int(count)) => Err(format!(
            "http_serve: stream response.max_events must be positive, got {count}"
        )),
        Some(other) => Err(format!(
            "http_serve: stream response.max_events must be int, got {}",
            other.type_name()
        )),
    }
}

/// Read `max_duration_ms`, defaulting and clamping as documented.
///
/// # Arguments
///
/// * `map` — Borrowed streaming-response map.
///
/// # Returns
///
/// The lifetime cap in milliseconds, at most [`DURATION_CEILING_MS`].
///
/// # Errors
///
/// Returns `Err` naming `max_duration_ms` when the value is not a positive int.
pub(super) fn duration_ms(map: &HashMap<String, Value>) -> Result<u64, String> {
    match map.get("max_duration_ms") {
        None | Some(Value::Nil) => Ok(DEFAULT_MAX_DURATION_MS),
        Some(Value::Int(ms)) if *ms > 0 => Ok((*ms as u64).min(DURATION_CEILING_MS)),
        Some(Value::Int(ms)) => Err(format!(
            "http_serve: stream response.max_duration_ms must be positive, got {ms}"
        )),
        Some(other) => Err(format!(
            "http_serve: stream response.max_duration_ms must be int, got {}",
            other.type_name()
        )),
    }
}
