//! Expiry evaluation for a stored record.
//!
//! Kept apart from [`super::store_record`] so the data shape and the policy that
//! reads it are never edited as one blob: a backend implementing the seam stores
//! the fields verbatim and calls this to decide, rather than reimplementing the
//! rule and drifting from it.

use super::store_record::{Expiry, Record};

/// Decide whether a record has expired, and under which rule.
///
/// # Arguments
///
/// * `record` — The stored session.
/// * `now_ms` — Current Unix milliseconds, passed in so tests can move the clock.
///
/// # Returns
///
/// `None` when the session is still live, otherwise the rule that ended it.
/// Absolute lifetime is checked **first**: when both have lapsed, the hard
/// ceiling is the more informative answer, since an idle report would suggest
/// that touching sooner would have helped when nothing would have.
///
/// A non-positive TTL disables that rule rather than expiring everything
/// immediately, which is what lets a caller opt out of one clock while keeping
/// the other. Validation at creation already rejects negatives, so this is a
/// defence in depth for a backend that stored a bad value.
///
/// # Examples
///
/// ```rust,ignore
/// // Idle window of 1s, absolute of 1h; last seen 2s ago.
/// assert!(matches!(evaluate(&record, now), Some(Expiry::Idle)));
/// ```
pub(super) fn evaluate(record: &Record, now_ms: i64) -> Option<Expiry> {
    if record.absolute_ttl_ms > 0
        && now_ms.saturating_sub(record.created_ms) >= record.absolute_ttl_ms
    {
        return Some(Expiry::Absolute);
    }
    if record.idle_ttl_ms > 0 && now_ms.saturating_sub(record.seen_ms) >= record.idle_ttl_ms {
        return Some(Expiry::Idle);
    }
    None
}

/// Render the failure a caller sees when a session has expired.
///
/// # Arguments
///
/// * `label` — Built-in name, so the message says which call failed.
/// * `reason` — The rule that ended the session.
///
/// # Returns
///
/// A message naming both the call and the rule. It deliberately does **not**
/// echo the id: session ids land in logs, and a logged id is a usable credential
/// until it expires.
pub(super) fn message(label: &str, reason: &Expiry) -> String {
    match reason {
        Expiry::Idle => format!("{label}: session expired after its idle timeout"),
        Expiry::Absolute => format!("{label}: session expired at its absolute lifetime"),
    }
}
