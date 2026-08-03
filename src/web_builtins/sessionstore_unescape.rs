//! The inverse of [`super::sessionstore_escape::escape`].
//!
//! Kept in its own file because unescaping is the only half that can *fail*: an
//! encoded string may arrive from Redis after a partial write or a foreign writer,
//! and a trailing or unknown escape must be reported rather than guessed at. A
//! lenient unescaper that dropped an unknown `\x` would silently mutate session
//! data.

/// Reverse the escaping, rejecting malformed input.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in the error.
/// * `text` — Escaped component.
///
/// # Returns
///
/// The original raw text.
///
/// # Errors
///
/// Returns a named error when the text ends in a lone `\`, or when a `\` is
/// followed by anything other than `\`, `s`, `e`, `n`, or `r`.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(unescape("l", "a\\sb").unwrap(), "a;b");
/// assert!(unescape("l", "a\\").is_err());
/// assert!(unescape("l", "a\\q").is_err());
/// ```
pub(super) fn unescape(label: &str, text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| format!("{label}: text ends with a dangling `\\` escape"))?;
        out.push(decode_escape(label, escaped)?);
    }
    Ok(out)
}

/// Map one escape letter back to its character.
fn decode_escape(label: &str, letter: char) -> Result<char, String> {
    match letter {
        '\\' => Ok('\\'),
        's' => Ok(super::sessionstore_escape::ENTRY_SEP),
        'e' => Ok(super::sessionstore_escape::PAIR_SEP),
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        other => Err(format!("{label}: unknown escape `\\{other}`")),
    }
}
