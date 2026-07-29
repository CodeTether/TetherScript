//! Column-width measurement helper for line fitting tests.

use crate::interp::tui::width::char_width;

/// Measure display columns while stepping over ANSI escape sequences.
pub(super) fn visible_width(text: &str) -> usize {
    let mut total = 0;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            for esc in chars.by_ref() {
                if esc.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        total += char_width(ch);
    }
    total
}
