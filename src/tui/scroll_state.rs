//! Scroll and selected-row metadata parsing for widgets.

use std::collections::HashMap;

use crate::value::Value;

use super::fields;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ScrollState {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) selected: Option<usize>,
}

pub(super) fn parse(map: &HashMap<String, Value>) -> Result<ScrollState, String> {
    let y = if map.contains_key("scroll_y") {
        fields::usize_or(map, "scroll_y", 0, 0, i64::MAX)?
    } else {
        fields::usize_or(map, "scroll", 0, 0, i64::MAX)?
    };
    Ok(ScrollState {
        x: fields::usize_or(map, "scroll_x", 0, 0, i64::MAX)?,
        y,
        selected: selected(map)?,
    })
}

fn selected(map: &HashMap<String, Value>) -> Result<Option<usize>, String> {
    Ok(
        fields::maybe_usize(map, "selected", 0, i64::MAX)?.or(fields::maybe_usize(
            map,
            "selected_row",
            0,
            i64::MAX,
        )?),
    )
}
