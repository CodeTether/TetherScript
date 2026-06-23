//! Bordered panel rendering.

use super::{buffer::Buffer, line, view};

/// Render a parsed view as a bordered terminal panel.
pub(super) fn render(view: &view::View) -> String {
    let inner = view.width.saturating_sub(2);
    let mut out = Buffer::new();
    out.push(border(inner, &view.title));
    for row in 0..view.height.saturating_sub(2) {
        out.push(body(inner, view.lines.get(row).map_or("", String::as_str)));
    }
    out.push(border(inner, &view.status));
    out.finish()
}

fn body(inner: usize, text: &str) -> String {
    format!("|{}|", line::fit(text, inner))
}

fn border(inner: usize, label: &str) -> String {
    if label.is_empty() {
        return format!("+{}+", "-".repeat(inner));
    }
    let label = line::fit(&format!(" {label} "), inner);
    format!("+{label}+")
}
