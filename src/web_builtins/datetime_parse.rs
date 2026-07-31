//! RFC 3339 parsing and the group's parse entry points.

use super::datetime_fields::{build, clock, number};

pub(super) use super::datetime_http::http_date_parse;

/// Parse an RFC 3339 timestamp into Unix seconds.
///
/// # Arguments
///
/// * `text` — A timestamp such as `2015-10-21T07:28:00Z`.
///
/// # Returns
///
/// Unix seconds for the parsed instant. Fractional seconds are accepted and
/// truncated, since Unix seconds carry no sub-second precision.
///
/// # Errors
///
/// Returns an error naming the problem: a missing date/time separator, a
/// component that is not numeric or is out of range, or a zone other than `Z`.
/// A numeric offset such as `+02:00` is rejected rather than silently read as
/// UTC, which would shift the instant.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(rfc3339_parse("1970-01-01T00:00:00Z"), Ok(0));
/// ```
pub(super) fn rfc3339_parse(text: &str) -> Result<i64, String> {
    let text = text.trim();
    let (date, rest) = text
        .split_once(['T', 't', ' '])
        .ok_or_else(|| format!("rfc3339_parse: missing the `T` separator in `{text}`"))?;
    let time = rest
        .strip_suffix('Z')
        .or_else(|| rest.strip_suffix('z'))
        .ok_or_else(|| {
            format!(
                "rfc3339_parse: expected the UTC marker `Z`; \
                 numeric offsets are unsupported, got `{rest}`"
            )
        })?;

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(format!(
            "rfc3339_parse: expected `YYYY-MM-DD`, got `{date}`"
        ));
    }
    let time = time.split_once('.').map_or(time, |(head, _)| head);
    let (hour, minute, second) = clock(time)?;
    build(
        number(parts[0], "year")?,
        number(parts[1], "month")?,
        number(parts[2], "day")?,
        hour,
        minute,
        second,
    )
}
