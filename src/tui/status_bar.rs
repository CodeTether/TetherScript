//! Status bar metadata parsing for terminal views.

use std::collections::HashMap;

use crate::value::Value;

use super::{fields, val};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StatusBar {
    pub(super) text: String,
    pub(super) kind: String,
    pub(super) busy: bool,
}

pub(super) fn parse(root: &HashMap<String, Value>) -> Result<StatusBar, String> {
    match root.get("status") {
        Some(Value::Map(raw)) => {
            let map = raw.borrow();
            from_map(&map)
        }
        Some(value) => Ok(StatusBar {
            text: value.to_string(),
            kind: String::new(),
            busy: false,
        }),
        None => Ok(empty()),
    }
}

fn from_map(map: &HashMap<String, Value>) -> Result<StatusBar, String> {
    let kind = match fields::text(map, "kind").as_str() {
        "" => fields::text(map, "mode"),
        value => value.to_string(),
    };
    Ok(StatusBar {
        text: fields::text(map, "text"),
        kind,
        busy: match map.get("busy") {
            Some(value) => val::bool_arg(value, "busy")?,
            None => false,
        },
    })
}

fn empty() -> StatusBar {
    StatusBar {
        text: String::new(),
        kind: String::new(),
        busy: false,
    }
}
