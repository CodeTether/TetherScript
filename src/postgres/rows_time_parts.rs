//! Field splitting for text-format timestamps.
//!
//! Split from [`super::rows_time`] so the arithmetic and the string handling stay separable.

/// Split a trailing zone offset from the clock, returning the offset in seconds.
///
/// The sign search starts after position 0 so a leading `-` cannot be mistaken for an offset.
pub(super) fn split_offset(rest: &str) -> (&str, i64) {
    let Some(index) = rest.get(1..).and_then(|tail| tail.find(['+', '-'])) else {
        return (rest, 0);
    };
    let (clock, zone) = rest.split_at(index + 1);
    (clock, offset_seconds(zone).unwrap_or(0))
}

/// Parse `+HH`, `+HHMM`, or `+HH:MM` into seconds east of UTC.
fn offset_seconds(zone: &str) -> Option<i64> {
    let mut chars = zone.chars();
    let sign = match chars.next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let digits: String = chars.filter(char::is_ascii_digit).collect();
    let hours: i64 = digits.get(0..2)?.parse().ok()?;
    let minutes: i64 = match digits.get(2..4) {
        Some(text) => text.parse().ok()?,
        None => 0,
    };
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Parse `YYYY-MM-DD`.
pub(super) fn date_parts(text: &str) -> Option<(i64, i64, i64)> {
    let mut parts = text.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    Some((year, month, day))
}

/// Parse `HH:MM:SS` with optional fractional seconds, which are truncated.
pub(super) fn clock_parts(text: &str) -> Option<(i64, i64, i64)> {
    let whole = text.split('.').next()?;
    let mut parts = whole.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let second = match parts.next() {
        Some(text) => text.parse().ok()?,
        None => 0,
    };
    Some((hour, minute, second))
}
