//! Structured view parsing for `tui_render`.

use crate::value::Value;

use super::{line, val};

pub(super) struct View {
    pub width: usize,
    pub height: usize,
    pub title: String,
    pub status: String,
    pub lines: Vec<String>,
}

pub(super) fn parse(value: &Value) -> Result<View, String> {
    let map = val::map_arg(value, "tui_render: view")?;
    let width = int_field(&map, "width", 80, 10, 240)?;
    let height = int_field(&map, "height", 24, 4, 80)?;
    Ok(View {
        width,
        height,
        title: str_field(&map, "title"),
        status: str_field(&map, "status"),
        lines: lines(&map)?,
    })
}

fn int_field(
    map: &std::collections::HashMap<String, Value>,
    key: &str,
    default: usize,
    min: usize,
    max: usize,
) -> Result<usize, String> {
    match map.get(key) {
        Some(value) => Ok(val::int_arg(value, key)?.clamp(min as i64, max as i64) as usize),
        None => Ok(default),
    }
}

fn str_field(map: &std::collections::HashMap<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::Str(value)) => value.to_string(),
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

fn lines(map: &std::collections::HashMap<String, Value>) -> Result<Vec<String>, String> {
    match map.get("items").or_else(|| map.get("lines")) {
        Some(Value::List(items)) => Ok(items.borrow().iter().map(line::item).collect()),
        Some(other) => Err(format!(
            "tui_render: items must be list, got {}",
            other.type_name()
        )),
        None => Ok(Vec::new()),
    }
}
