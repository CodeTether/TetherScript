//! Quote-aware splitting for filter pipelines and argument lists.
//!
//! A separator inside a quoted literal is data, not structure: `date(format="%b %d, %Y")`
//! contains both a comma and, in other views, a `|`. Splitting naively truncated the
//! pattern at the first comma and silently rendered a partial date.

/// Split `text` on `separator`, ignoring separators inside single or double quotes.
///
/// # Arguments
///
/// * `text` — Source to split.
/// * `separator` — Byte to split on, such as `|` or `,`.
///
/// # Returns
///
/// The segments, with quotes preserved so the caller can still unquote literals.
pub(super) fn split_outside_quotes(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote: Option<char> = None;
    let mut start = 0usize;
    for (index, character) in text.char_indices() {
        match quote {
            // Inside a literal, only the matching quote is structural.
            Some(open) if character == open => quote = None,
            Some(_) => {}
            None if character == '"' || character == '\'' => quote = Some(character),
            None if character == separator => {
                parts.push(&text[start..index]);
                start = index + character.len_utf8();
            }
            None => {}
        }
    }
    parts.push(&text[start..]);
    parts
}
