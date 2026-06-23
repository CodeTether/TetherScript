//! Styled text span parsing for future render integration.

use crate::value::Value;

use super::{style, style::Style, style_attr, val};

#[derive(Clone)]
pub(super) struct Span {
    pub(super) text: String,
    pub(super) style: Style,
}

pub(super) fn parse(value: &Value) -> Result<Span, String> {
    let map = val::map_arg(value, "tui_span: span")?;
    let text = style_attr::required_text(&map, "text", "tui_span")?;
    let style = match map.get("style") {
        Some(value) => Style::parse(value)?,
        None => Style::from_fields(&map)?,
    };
    Ok(Span { text, style })
}

pub(super) fn render(value: &Value) -> Result<String, String> {
    let span = parse(value)?;
    style::paint(&span.text, &span.style)
}

pub(super) fn render_value(args: &[Value]) -> Result<Value, String> {
    let value = args.first().ok_or("tui_span_render: missing span")?;
    Ok(val::strv(render(value)?))
}
