//! Clock-free keepalive policy.
//!
//! Intermediaries close a connection that has been silent too long, and an SSE
//! stream is legitimately silent whenever nothing has happened. A periodic comment
//! line keeps bytes moving without dispatching an event at the client.
//!
//! **This module never reads the clock.** Both times arrive as arguments, so the
//! decision is a pure function and every boundary — below, exactly at, and past
//! the interval — is testable without sleeping. The caller supplies the clock,
//! which also lets a test drive a fake one.

/// Whether a keepalive comment is due.
///
/// # Arguments
///
/// * `now_ms` — Current time in milliseconds, on any monotonic scale.
/// * `last_write_ms` — Same scale, when the last byte was written. A keepalive
///   counts as a write, so the caller resets this after emitting one.
/// * `interval_ms` — Maximum silence permitted. An `interval_ms` of `0` makes a
///   keepalive due on every check.
///
/// # Returns
///
/// `true` when `now_ms - last_write_ms >= interval_ms`. The comparison is `>=`, so
/// **exactly** at the interval a keepalive is due: waiting for one more millisecond
/// would mean a proxy with an equal timeout can win the race.
///
/// A `now_ms` earlier than `last_write_ms` yields `false` via saturating
/// subtraction. That can only happen if a non-monotonic clock stepped backwards,
/// and an extra comment line is cheaper than a panic in a server loop.
///
/// # Panics
///
/// Never. Subtraction saturates.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::keepalive::is_due;
///
/// assert!(!is_due(1_000, 0, 15_000));      // below the interval
/// assert!(is_due(15_000, 0, 15_000));      // exactly at it: due
/// assert!(is_due(15_001, 0, 15_000));      // past it
/// assert!(!is_due(0, 15_000, 15_000));     // clock went backwards
/// ```
pub fn is_due(now_ms: u64, last_write_ms: u64, interval_ms: u64) -> bool {
    now_ms.saturating_sub(last_write_ms) >= interval_ms
}

/// Milliseconds remaining before the next keepalive is due.
///
/// Lets a server loop pick a poll timeout instead of spinning.
///
/// # Arguments
///
/// * `now_ms` — Current time in milliseconds.
/// * `last_write_ms` — When the last byte was written, same scale.
/// * `interval_ms` — Maximum silence permitted.
///
/// # Returns
///
/// `0` exactly when [`is_due`] returns `true`; otherwise the remaining wait.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::keepalive::{is_due, remaining_ms};
///
/// assert_eq!(remaining_ms(1_000, 0, 15_000), 14_000);
/// assert_eq!(remaining_ms(15_000, 0, 15_000), 0);
/// assert!(is_due(15_000, 0, 15_000));
/// ```
pub fn remaining_ms(now_ms: u64, last_write_ms: u64, interval_ms: u64) -> u64 {
    interval_ms.saturating_sub(now_ms.saturating_sub(last_write_ms))
}

/// A sane default interval: 15 seconds.
///
/// Comfortably under the common 30- and 60-second idle timeouts in nginx and most
/// load balancers, while costing seven bytes per client per interval.
pub const DEFAULT_INTERVAL_MS: u64 = 15_000;
