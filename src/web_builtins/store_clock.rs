//! The clock every session lifetime is measured against.
//!
//! Milliseconds, not seconds. The signed-cookie half in [`super::super::session`]
//! works in Unix *seconds* because a cookie `exp` is defined that way on the wire.
//! A server-side record has no wire format, and second granularity would make an
//! idle timeout of "one second" indistinguishable from zero after truncation, so
//! the store keeps millisecond resolution throughout.
//!
//! Derived from [`crate::system::time_now_ms`], the same clock the `time_now_ms`
//! builtin exposes, so a script comparing its own timestamps against a session's
//! cannot see the two disagree.

use crate::value::Value;

/// Current wall-clock time in Unix milliseconds.
///
/// # Returns
///
/// Milliseconds since the Unix epoch, or `0` if the host clock is before the
/// epoch — the same degenerate answer `time_now_ms` gives, so expiry arithmetic
/// stays total rather than panicking on a broken clock.
///
/// # Examples
///
/// ```rust,ignore
/// let start = now_ms();
/// assert!(now_ms() >= start);
/// ```
pub(super) fn now_ms() -> i64 {
    match crate::system::time_now_ms() {
        Value::Int(millis) => millis,
        _ => 0,
    }
}
