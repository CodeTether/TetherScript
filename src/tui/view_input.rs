//! Input widget rows folded into simple frames.

use std::collections::HashMap;

use crate::value::Value;

use super::input_state;

pub(super) fn lines(map: &HashMap<String, Value>) -> Result<Vec<String>, String> {
    let Some(input) = input_state::parse(map)? else {
        return Ok(Vec::new());
    };
    let focus = if input.focused { "*" } else { " " };
    let text = if input.text.is_empty() {
        input.placeholder
    } else {
        input.text
    };
    Ok(vec![format!(
        "{focus} {}{} @{}",
        input.prompt, text, input.cursor
    )])
}
