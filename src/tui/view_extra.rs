//! Extra structured view metadata folded into simple frames.

use std::collections::HashMap;

use crate::value::Value;

use super::{panel_state, status_bar, view_input};

pub(super) fn status(map: &HashMap<String, Value>) -> Result<String, String> {
    let status = status_bar::parse(map)?;
    if status.busy && !status.kind.is_empty() {
        return Ok(format!("{} {} ...", status.kind, status.text));
    }
    if status.kind.is_empty() {
        Ok(status.text)
    } else {
        Ok(format!("{} {}", status.kind, status.text))
    }
}

pub(super) fn merge(
    mut rows: Vec<String>,
    map: &HashMap<String, Value>,
) -> Result<Vec<String>, String> {
    rows.extend(panel_lines(map)?);
    rows.extend(view_input::lines(map)?);
    Ok(rows)
}

fn panel_lines(map: &HashMap<String, Value>) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for panel in panel_state::parse_all(map)? {
        out.push(format!(
            "[panel {} @{},{} {}x{} sx={}] {}",
            panel.id, panel.x, panel.y, panel.width, panel.height, panel.scroll.x, panel.title
        ));
        for (index, row) in panel.rows.iter().skip(panel.scroll.y).enumerate() {
            let selected = panel.scroll.selected == Some(panel.scroll.y + index);
            out.push(format!("{} {}", if selected { ">" } else { " " }, row));
        }
    }
    Ok(out)
}
