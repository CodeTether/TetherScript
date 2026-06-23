//! ANSI escape scanning for fitted terminal lines.

/// Copy one ANSI escape sequence into `out`.
pub(super) fn push(out: &mut String, chars: &mut std::str::Chars<'_>) {
    out.push('\x1b');
    for ch in chars.by_ref() {
        out.push(ch);
        if ch.is_ascii_alphabetic() {
            break;
        }
    }
}
