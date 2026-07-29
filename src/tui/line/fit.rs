//! Line shaping for terminal frames.

use crate::value::Value;

use super::item as line_item;
use crate::interp::tui::{escape, width};

/// Fit `text` to exactly `width` terminal columns.
///
/// Measurement uses display columns, not character count, so wide (CJK,
/// emoji) and zero-width (combining mark) characters keep frame borders
/// aligned. A wide character that would straddle the right edge is
/// replaced by a single space so the column total stays exact.
pub(crate) fn fit(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut visible = 0;
    let mut truncated = false;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            escape::push(&mut out, &mut chars);
            continue;
        }
        let cols = width::char_width(ch);
        if visible + cols > width {
            // A wide glyph straddling the edge cannot be split; pad instead.
            truncated = true;
            break;
        }
        out.push(ch);
        visible += cols;
    }
    if truncated && text.contains('\x1b') {
        out.push_str("\x1b[0m");
    }
    while visible < width {
        out.push(' ');
        visible += 1;
    }
    out
}

/// Render a script value as a single terminal row.
pub(crate) fn item(value: &Value) -> String {
    line_item::render(value)
}
