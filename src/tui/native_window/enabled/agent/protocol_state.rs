//! Agent session-state response decoding.

use std::collections::HashMap;

use crate::value::Value;

use super::super::model::{Event, Message};

pub(super) fn decode(result: &HashMap<String, Value>) -> Option<Event> {
    let Value::List(values) = result.get("messages")? else {
        return None;
    };
    let messages = values.borrow().iter().filter_map(message).collect();
    Some(Event::State {
        messages,
        model: field(result, "model"),
        workspace: field(result, "workspace"),
        ready: matches!(result.get("provider_ready"), Some(Value::Bool(true))),
    })
}

fn message(value: &Value) -> Option<Message> {
    let Value::Map(map) = value else {
        return None;
    };
    let map = map.borrow();
    Some(Message {
        role: field(&map, "role"),
        text: field(&map, "content"),
    })
}

fn field(map: &HashMap<String, Value>, key: &str) -> String {
    map.get(key).map(Value::to_string).unwrap_or_default()
}
