//! Panel data parsing for richer terminal layouts.

use std::collections::HashMap;

use crate::value::Value;

use super::{fields, panel_rows, scroll_state};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PanelState {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) rows: Vec<String>,
    pub(super) scroll: scroll_state::ScrollState,
}

pub(super) fn parse_all(root: &HashMap<String, Value>) -> Result<Vec<PanelState>, String> {
    fields::list(root, "panels", "tui view: panels")?
        .iter()
        .enumerate()
        .map(|(index, value)| parse_one(value, index))
        .collect()
}

fn parse_one(value: &Value, index: usize) -> Result<PanelState, String> {
    match value {
        Value::Map(raw) => {
            let map = raw.borrow();
            Ok(PanelState {
                id: fields::text(&map, "id"),
                title: fields::text(&map, "title"),
                x: fields::usize_or(&map, "x", 0, 0, 500)?,
                y: fields::usize_or(&map, "y", 0, 0, 500)?,
                width: fields::usize_or(&map, "width", 0, 0, 500)?,
                height: fields::usize_or(&map, "height", 0, 0, 500)?,
                rows: panel_rows::parse(&map, index)?,
                scroll: scroll_state::parse(&map)?,
            })
        }
        other => Err(format!(
            "tui view: panels[{index}] must be map, got {}",
            other.type_name()
        )),
    }
}
