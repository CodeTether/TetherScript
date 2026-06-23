use crate::value::Value;

pub(super) fn role(value: &Value) -> Option<String> {
    string(value, "role")
}

pub(super) fn content(value: &Value) -> Option<String> {
    string(value, "content")
}

pub(super) fn call_id(value: &Value) -> Option<String> {
    string(value, "tool_call_id")
        .or_else(|| string(value, "call_id"))
        .or_else(|| string(value, "id"))
}

pub(super) fn tool_calls(value: &Value) -> Vec<Value> {
    list(value, "tool_calls")
}

pub(super) fn function(value: &Value) -> Option<Value> {
    field(value, "function")
}

pub(super) fn name(value: &Value) -> Option<String> {
    string(value, "name")
}

pub(super) fn arguments(value: &Value) -> Option<String> {
    string(value, "arguments")
}

fn string(value: &Value, key: &str) -> Option<String> {
    match field(value, key) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    }
}

fn list(value: &Value, key: &str) -> Vec<Value> {
    match field(value, key) {
        Some(Value::List(list)) => list.borrow().clone(),
        _ => Vec::new(),
    }
}

fn field(value: &Value, key: &str) -> Option<Value> {
    let Value::Map(map) = value else {
        return None;
    };
    map.borrow().get(key).cloned()
}
