//! Parsed TUI style maps and script-facing style helpers.

use std::collections::HashMap;

use crate::value::Value;

use super::{style_attr, style_open, val};

#[derive(Clone, Default)]
pub(super) struct Style {
    pub(super) fg: Option<String>,
    pub(super) bg: Option<String>,
    pub(super) bold: bool,
    pub(super) dim: bool,
    pub(super) underline: bool,
    pub(super) inverse: bool,
}

impl Style {
    pub(super) fn parse(value: &Value) -> Result<Self, String> {
        let map = val::map_arg(value, "tui_style: style")?;
        Self::from_fields(&map)
    }

    pub(super) fn from_fields(map: &HashMap<String, Value>) -> Result<Self, String> {
        Ok(Self {
            fg: style_attr::text_field(map, "fg")?,
            bg: style_attr::text_field(map, "bg")?,
            bold: style_attr::bool_field(map, "bold")?,
            dim: style_attr::bool_field(map, "dim")?,
            underline: style_attr::bool_field(map, "underline")?,
            inverse: style_attr::bool_field(map, "inverse")?,
        })
    }

    pub(super) fn open(&self) -> Result<String, String> {
        style_open::open(self)
    }
}

pub(super) fn open_value(args: &[Value]) -> Result<Value, String> {
    let value = args.first().ok_or("tui_style_open: missing style")?;
    Ok(val::strv(Style::parse(value)?.open()?))
}

pub(super) fn reset_value(_: &[Value]) -> Result<Value, String> {
    Ok(val::strv(reset()))
}

pub(super) fn reset() -> &'static str {
    "\x1b[0m"
}

pub(super) fn paint(text: &str, style: &Style) -> Result<String, String> {
    let open = style.open()?;
    if open.is_empty() {
        Ok(text.to_string())
    } else {
        Ok(format!("{open}{text}{}", reset()))
    }
}
