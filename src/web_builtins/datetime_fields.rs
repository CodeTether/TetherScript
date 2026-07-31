//! Numeric field parsing shared by both date parsers.
//!
//! Both formats are fixed-shape, so a component that is not plain decimal digits
//! is an error rather than something to coerce. Naming the field and the offending
//! text is what makes a malformed `Expires` header diagnosable.

/// Validate every component, then convert to Unix seconds.
///
/// # Arguments
///
/// * `year`, `month`, `day` — Civil date components.
/// * `hour`, `minute`, `second` — Time of day.
///
/// # Returns
///
/// Unix seconds for the instant.
///
/// # Errors
///
/// Returns an error naming the out-of-range component, including a day that does
/// not exist in that month.
pub(super) fn build(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> Result<i64, String> {
    let last = super::datetime_month::valid_day(year, month);
    if last == 0 {
        return Err(format!("date parse: month {month} is outside 1-12"));
    }
    if day < 1 || day > last {
        return Err(format!(
            "date parse: day {day} is outside 1-{last} for month {month} of {year}"
        ));
    }
    if hour > 23 || minute > 59 || second > 60 {
        return Err(format!(
            "date parse: time {hour:02}:{minute:02}:{second:02} is out of range"
        ));
    }
    // A leap second (:60) is clamped rather than rejected: servers do send it and
    // Unix time has no representation for it.
    let second = second.min(59);
    Ok(
        super::datetime_civil::days_from_civil(year, month, day) * 86_400
            + hour * 3600
            + minute * 60
            + second,
    )
}

/// Parse an `HH:MM:SS` group.
///
/// # Errors
///
/// Returns an error when the group does not have exactly three numeric parts.
pub(super) fn clock(text: &str) -> Result<(i64, i64, i64), String> {
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 3 {
        return Err(format!("date parse: expected `HH:MM:SS`, got `{text}`"));
    }
    Ok((
        number(parts[0], "hour")?,
        number(parts[1], "minute")?,
        number(parts[2], "second")?,
    ))
}

/// Parse one all-digit component.
///
/// # Errors
///
/// Returns an error naming the field when the text is empty, contains a
/// non-digit, or overflows.
pub(super) fn number(text: &str, field: &str) -> Result<i64, String> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "date parse: {field} `{text}` is not a decimal number"
        ));
    }
    text.parse::<i64>()
        .map_err(|_| format!("date parse: {field} `{text}` does not fit"))
}
