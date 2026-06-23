//! ASCII line shaping for terminal frames.

use crate::value::Value;

use super::{escape, line_item};

pub(super) fn fit(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut visible = 0;
    let mut truncated = false;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            escape::push(&mut out, &mut chars);
        } else if visible < width {
            out.push(ch);
            visible += 1;
        } else {
            truncated = true;
            break;
        }
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

pub(super) fn item(value: &Value) -> String {
    line_item::render(value)
}
