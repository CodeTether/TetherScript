//! Render structured terminal views.

use crate::output;
use crate::value::Value;

use super::{line, val, view};

pub(super) fn render(args: &[Value]) -> Result<Value, String> {
    Ok(val::strv(frame(&view::parse(&args[0])?)))
}

pub(super) fn present(args: &[Value]) -> Result<Value, String> {
    let frame = frame(&view::parse(&args[0])?);
    output::write("\x1b[?25l\x1b[2J\x1b[H");
    output::write(&frame);
    output::write("\x1b[?25h");
    Ok(Value::Nil)
}

fn frame(view: &view::View) -> String {
    let inner = view.width.saturating_sub(2);
    let mut out = String::new();
    out.push_str(&border(inner, &view.title));
    let body_rows = view.height.saturating_sub(3);
    for row in 0..body_rows {
        out.push('|');
        out.push_str(&line::fit(
            view.lines.get(row).map_or("", String::as_str),
            inner,
        ));
        out.push_str("|\n");
    }
    out.push_str(&border(inner, &view.status));
    out
}

fn border(inner: usize, label: &str) -> String {
    if label.is_empty() {
        return format!("+{}+\n", "-".repeat(inner));
    }
    let visible = format!(" {label} ");
    let label: String = visible.chars().take(inner).collect();
    let fill = "-".repeat(inner.saturating_sub(label.len()));
    format!("+{label}{fill}+\n")
}
