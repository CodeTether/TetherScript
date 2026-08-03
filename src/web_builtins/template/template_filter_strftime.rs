//! strftime rendering for the `date` filter.
//!
//! Supports the specifiers the reference views actually use. An unknown specifier is
//! emitted verbatim rather than dropped, so a typo is visible in the output instead of
//! silently vanishing.

use super::template_filter_month::MONTHS;

/// Render `seconds` according to `pattern`.
pub(super) fn render(seconds: i64, pattern: &str) -> String {
    let (year, month, day) =
        super::template_filter_civil::civil_from_days(seconds.div_euclid(86400));
    let rest = seconds.rem_euclid(86400);
    let (hour, minute, second) = (rest / 3600, (rest % 3600) / 60, rest % 60);

    let mut out = String::with_capacity(pattern.len() + 16);
    let mut chars = pattern.chars();
    while let Some(character) = chars.next() {
        if character != '%' {
            out.push(character);
            continue;
        }
        let index = (month - 1) as usize;
        match chars.next() {
            Some('Y') => out.push_str(&year.to_string()),
            Some('m') => out.push_str(&format!("{month:02}")),
            Some('d') => out.push_str(&format!("{day:02}")),
            // %e is day-of-month space-padded, which the reference uses.
            Some('e') => out.push_str(&format!("{day:2}")),
            Some('b') => out.push_str(MONTHS[index].0),
            Some('B') => out.push_str(MONTHS[index].1),
            Some('H') => out.push_str(&format!("{hour:02}")),
            Some('M') => out.push_str(&format!("{minute:02}")),
            Some('S') => out.push_str(&format!("{second:02}")),
            Some('I') => out.push_str(&format!("{:02}", twelve_hour(hour))),
            Some('p') => out.push_str(if hour < 12 { "AM" } else { "PM" }),
            Some('%') => out.push('%'),
            // Unknown specifiers survive verbatim so a typo is visible.
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }
    out
}

/// Convert a 24-hour hour to its 12-hour form, where midnight and noon are both 12.
fn twelve_hour(hour: i64) -> i64 {
    match hour % 12 {
        0 => 12,
        other => other,
    }
}
