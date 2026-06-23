//! Responses tool option conversion.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

#[path = "tool_convert.rs"]
mod tool_convert;

pub(super) fn push(parts: &mut Vec<String>, opts: Option<&Value>) -> Result<(), String> {
    let map = match opts {
        Some(Value::Map(map)) => map.borrow(),
        _ => {
            parts.push("\"tool_choice\":\"auto\"".into());
            return Ok(());
        }
    };
    if let Some(tools) = map.get("tools") {
        parts.push(format!("\"tools\":{}", tools_json(tools)?));
    }
    parts.push(format!("\"tool_choice\":{}", choice_json(&map)?));
    if let Some(value) = map.get("parallel_tool_calls") {
        parts.push(format!(
            "\"parallel_tool_calls\":{}",
            crate::json::encode_to_string(value)?
        ));
    }
    Ok(())
}

fn choice_json(map: &std::collections::HashMap<String, Value>) -> Result<String, String> {
    match map.get("tool_choice") {
        Some(value @ Value::Map(_)) => {
            crate::json::encode_to_string(&tool_convert::convert(value)?)
        }
        Some(value) => crate::json::encode_to_string(value),
        None => Ok("\"auto\"".into()),
    }
}

fn tools_json(value: &Value) -> Result<String, String> {
    let Value::List(items) = value else {
        return Err("provider.responses: tools must be a list".into());
    };
    let tools = items
        .borrow()
        .iter()
        .map(tool_convert::convert)
        .collect::<Result<Vec<_>, _>>()?;
    crate::json::encode_to_string(&Value::List(Rc::new(RefCell::new(tools))))
}
