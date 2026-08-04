//! Quote-aware splitting for filter pipelines and argument lists.

/// Split `text` on `separator`, ignoring separators inside single or double quotes.
pub(super) fn split_outside_quotes(text: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote: Option<char> = None;
    let mut start = 0usize;
    for (i, c) in text.char_indices() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == sep => {
                parts.push(&text[start..i]);
                start = i + c.len_utf8();
            }
            None => {}
        }
    }
    parts.push(&text[start..]);
    parts
}
