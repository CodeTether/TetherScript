//! Render structured terminal views.

use crate::output;
use crate::value::Value;

use super::{diff, panel, val, view};

pub(super) fn render(args: &[Value]) -> Result<Value, String> {
    Ok(val::strv(frame(&view::parse(&args[0])?)))
}

pub(super) fn present(args: &[Value]) -> Result<Value, String> {
    let frame = frame(&view::parse(&args[0])?);
    output::write(&diff::full_redraw(&frame));
    Ok(Value::Nil)
}

fn frame(view: &view::View) -> String {
    panel::render(view)
}
