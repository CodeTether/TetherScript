//! Input buffer metadata parsing for terminal widgets.

use std::collections::HashMap;

use crate::value::Value;

use super::{fields, val};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InputState {
    pub(super) prompt: String,
    pub(super) text: String,
    pub(super) cursor: usize,
    pub(super) focused: bool,
    pub(super) placeholder: String,
}

pub(super) fn parse(root: &HashMap<String, Value>) -> Result<Option<InputState>, String> {
    match root.get("input") {
        Some(Value::Map(raw)) => {
            let map = raw.borrow();
            Ok(Some(from_map(&map)?))
        }
        Some(other) => Err(format!(
            "tui view: input must be map, got {}",
            other.type_name()
        )),
        None => Ok(None),
    }
}

fn from_map(map: &HashMap<String, Value>) -> Result<InputState, String> {
    Ok(InputState {
        prompt: fields::text(map, "prompt"),
        text: fields::text(map, "text"),
        cursor: fields::usize_or(map, "cursor", 0, 0, i64::MAX)?,
        focused: match map.get("focused") {
            Some(value) => val::bool_arg(value, "focused")?,
            None => true,
        },
        placeholder: fields::text(map, "placeholder"),
    })
}
