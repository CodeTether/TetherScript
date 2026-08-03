//! # Parsing the time-of-day half of an ISO-8601 value
//!
//! `HH:MM[:SS[.ffffff]]` into microseconds since midnight.
//!
//! Two rules matter more than the framing:
//!
//! - **Fractional seconds are right-padded to six digits.** `.5` is 500 000 µs, not
//!   5 µs. Left-aligning, or parsing the fraction as a plain integer, would turn half
//!   a second into five microseconds — a 100 000× error that still looks like a valid
//!   timestamp.
//! - **More than six digits is rejected, not rounded.** Silently dropping precision
//!   from a timestamp is the same class of bug as decoding `numeric` through `f64`,
//!   so the caller is told instead.
//!
//! A leap second (`:60`) is rejected, matching PostgreSQL's own input rules.

use super::super::super::super::error::DecodeError;
use super::util::{bad, number};

/// Microseconds in one hour, and the multiplier chain used below.
const MICROS_PER_SECOND: i64 = 1_000_000;

/// Split `HH:MM[:SS[.ffffff]]` into microseconds since midnight.
///
/// # Arguments
///
/// * `text` — the trimmed time text.
///
/// # Returns
///
/// Microseconds since midnight, in `0..86_400_000_000`.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a malformed shape, an out-of-range field, or more
/// than six fractional digits.
pub(super) fn split_time(text: &str) -> Result<i64, DecodeError> {
    let parts: Vec<&str> = text.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(bad("time", text, "expected HH:MM[:SS[.ffffff]]"));
    }
    let hour: i64 = number(parts[0], "time", text)?;
    let minute: i64 = number(parts[1], "time", text)?;
    let (second, micros) = seconds(parts.get(2).copied().unwrap_or("0"), text)?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(bad("time", text, "hour<=23, minute<=59, second<=59"));
    }
    let whole = hour * 3_600 + minute * 60 + second;
    Ok(whole * MICROS_PER_SECOND + micros)
}

/// Split `SS` or `SS.ffffff`, right-padding the fraction to exactly six digits.
///
/// # Arguments
///
/// * `text` — the seconds field, with or without a fraction.
/// * `whole` — the full time text, quoted in any error message.
///
/// # Returns
///
/// `(whole_seconds, microseconds)`.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-numeric field or more than six fractional
/// digits.
fn seconds(text: &str, whole: &str) -> Result<(i64, i64), DecodeError> {
    let (whole_part, fraction) = text.split_once('.').unwrap_or((text, ""));
    if fraction.len() > 6 {
        return Err(bad("time", whole, "at most 6 fractional digits"));
    }
    let second: i64 = number(whole_part, "time", whole)?;
    if fraction.is_empty() {
        return Ok((second, 0));
    }
    // Right-pad so ".5" is 500_000 µs rather than 5 µs.
    let padded = format!("{fraction:0<6}");
    Ok((second, number(&padded, "time", whole)?))
}
