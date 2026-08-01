//! Token-bucket state: field access and the elapsed-time refill.
//!
//! The bucket is an ordinary tetherscript map, so the caller owns it and this
//! module stays free of shared mutable state.

use std::time::{SystemTime, UNIX_EPOCH};

/// Field names on the bucket map, kept in one place so the reader and the writer
/// cannot disagree about a spelling.
pub(super) const CAPACITY: &str = "capacity";
pub(super) const TOKENS: &str = "tokens";
pub(super) const REFILL: &str = "refill_per_sec";
pub(super) const UPDATED: &str = "updated_ms";

/// Current wall clock in Unix milliseconds.
///
/// Milliseconds rather than seconds because a refill computed from whole seconds
/// would round every sub-second gap down to no refill at all.
pub(super) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

/// Tokens available at `now`, given the tokens held at `updated`.
///
/// # Arguments
///
/// * `tokens` — Tokens remaining as of `updated`.
/// * `capacity` — Ceiling the bucket may never exceed.
/// * `refill_per_sec` — Tokens restored per second.
/// * `updated` — When `tokens` was last accurate, in Unix milliseconds.
/// * `now` — Current Unix milliseconds.
///
/// # Returns
///
/// The refilled token count, clamped to `capacity`.
///
/// Refill is derived from elapsed time, never from a call count, so a caller that
/// polls twice as often does not earn tokens twice as fast. The clamp is what
/// stops an idle client from banking an unbounded burst; without it a bucket idle
/// for a day would admit a day's worth of traffic at once.
pub(super) fn refilled(
    tokens: f64,
    capacity: f64,
    refill_per_sec: f64,
    updated: i64,
    now: i64,
) -> f64 {
    // A clock that moved backwards must not mint tokens.
    let elapsed_ms = (now - updated).max(0) as f64;
    let gained = elapsed_ms / 1000.0 * refill_per_sec;
    (tokens + gained).min(capacity)
}

/// Milliseconds until `needed` tokens exist, rounded up to a whole millisecond.
///
/// # Returns
///
/// At least 1 when a wait is required, so a denied caller never receives a
/// `retry_after_ms` of zero that invites an immediate retry.
pub(super) fn wait_ms(needed: f64, refill_per_sec: f64) -> i64 {
    if needed <= 0.0 {
        return 0;
    }
    let ms = needed / refill_per_sec * 1000.0;
    (ms.ceil() as i64).max(1)
}
