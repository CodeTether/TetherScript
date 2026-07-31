//! IMF-fixdate parsing (RFC 7231 preferred form).

use super::datetime_fields::{build, clock, number};
use super::datetime_month::month_from_name;

/// Parse an RFC 7231 IMF-fixdate into Unix seconds.
///
/// # Arguments
///
/// * `text` — A date such as `Wed, 21 Oct 2015 07:28:00 GMT`.
///
/// # Returns
///
/// Unix seconds for the parsed instant.
///
/// # Errors
///
/// Returns an error naming the problem: wrong field count, a non-numeric or
/// out-of-range component, an unknown month abbreviation, a day that does not
/// exist in that month, or a zone other than `GMT`.
///
/// The weekday is deliberately not validated against the date. Clients in the
/// wild send stale weekdays, and rejecting an otherwise unambiguous date would
/// lose a usable `Expires` value.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(http_date_parse("Thu, 01 Jan 1970 00:00:00 GMT"), Ok(0));
/// ```
pub(super) fn http_date_parse(text: &str) -> Result<i64, String> {
    let text = text.trim();
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(format!(
            "http_date_parse: expected 6 fields like \
             `Wed, 21 Oct 2015 07:28:00 GMT`, got {} in `{text}`",
            fields.len()
        ));
    }
    if !fields[5].eq_ignore_ascii_case("GMT") {
        return Err(format!(
            "http_date_parse: expected the zone `GMT`, got `{}`",
            fields[5]
        ));
    }
    let (hour, minute, second) = clock(fields[4])?;
    build(
        number(fields[3], "year")?,
        month_from_name(fields[2])?,
        number(fields[1], "day")?,
        hour,
        minute,
        second,
    )
}
