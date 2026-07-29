//! Agent JSON-RPC request and response conversion.

#[path = "protocol_request.rs"]
mod request_encode;
#[path = "protocol_state.rs"]
mod state_decode;

use std::collections::HashMap;

use crate::value::Value;

use super::model::Event;

pub(super) use request_encode::encode as request;

pub(super) fn decode(line: &str) -> Option<Event> {
    let Value::Map(root) = crate::json::parse_str(line).ok()? else {
        return None;
    };
    let root = root.borrow();
    if let Some(error) = root.get("error") {
        return Some(Event::Error(error.to_string()));
    }
    let Value::Map(result) = root.get("result")? else {
        return None;
    };
    let result = result.borrow();
    if result.contains_key("messages") {
        return state_decode::decode(&result);
    }
    let Value::List(content) = result.get("content")? else {
        return None;
    };
    let first = content.borrow().first()?.clone();
    let Value::Map(part) = first else { return None };
    Some(Event::Reply(field(&part.borrow(), "text")))
}

fn field(map: &HashMap<String, Value>, key: &str) -> String {
    map.get(key).map(Value::to_string).unwrap_or_default()
}
