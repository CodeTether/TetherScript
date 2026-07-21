//! Native HTML/CSS projection of the shared TUI view schema.

use crate::value::Value;

use super::view;

pub(crate) fn document(value: &Value) -> Result<(String, String), String> {
    let view = view::parse(value)?;
    let rows = view
        .lines
        .iter()
        .map(|line| format!("<p>{}</p>", escape(line)))
        .collect::<String>();
    let html = format!(
        "<main><header>{}</header><section>{rows}</section><footer>{}</footer></main>",
        escape(&view.title),
        escape(&view.status)
    );
    Ok((html, stylesheet(view.width, view.height)))
}

fn stylesheet(width: usize, height: usize) -> String {
    let pixel_width = width.saturating_mul(8);
    let pixel_height = height.saturating_mul(16);
    let body_height = height.saturating_sub(2).saturating_mul(16);
    format!(
        "main {{ width: {pixel_width}px; height: {pixel_height}px; background: #101820; color: #e8f0f8 }} header {{ height: 16px; background: #204060; color: #66ccff }} section {{ height: {body_height}px; background: #182838 }} p {{ height: 16px }} footer {{ height: 16px; background: #081018; color: #80e080 }}"
    )
}

fn escape(value: &str) -> String {
    value.chars().fold(String::new(), |mut output, character| {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(character),
        }
        output
    })
}
