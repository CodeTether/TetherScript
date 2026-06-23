//! Row extraction for structured panel maps.

use std::collections::HashMap;

use crate::value::Value;

use super::{fields, line};

pub(super) fn parse(map: &HashMap<String, Value>, index: usize) -> Result<Vec<String>, String> {
    let key = if map.contains_key("items") {
        "items"
    } else {
        "lines"
    };
    Ok(
        fields::list(map, key, &format!("tui view: panels[{index}].{key}"))?
            .iter()
            .map(line::item)
            .collect(),
    )
}
