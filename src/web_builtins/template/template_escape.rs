//! HTML escaping.
//!
//! Ordering is load-bearing: `&` must be replaced first, or the ampersands this
//! function itself introduces get escaped again and `<` renders as `&amp;lt;`
//! instead of `&lt;`.

/// Escape the five characters that are unsafe in HTML text or attributes.
///
/// # Arguments
///
/// * `text` — Untrusted text to escape.
///
/// # Returns
///
/// The text with `&`, `<`, `>`, `"`, and `'` replaced by named or numeric
/// entities. `'` becomes `&#39;` rather than `&apos;`, which older HTML parsers
/// do not recognize.
pub(super) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// Escape text for use inside a quoted attribute value.
///
/// # Arguments
///
/// * `text` — Untrusted attribute value.
///
/// # Returns
///
/// The escaped value. This additionally encodes the characters that let a value
/// break out of an unquoted attribute — tab, newline, carriage return, space,
/// `/`, `=`, and backtick — so the result stays inert even if a caller forgets
/// the surrounding quotes.
pub(super) fn escape_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '/' => out.push_str("&#47;"),
            '=' => out.push_str("&#61;"),
            '`' => out.push_str("&#96;"),
            '\t' => out.push_str("&#9;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            ' ' => out.push_str("&#32;"),
            other => out.push(other),
        }
    }
    out
}
