//! Field helpers for Responses output values.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn field(value: &Value, key: &str) -> Option<Value> {
    map(value)?.borrow().get(key).cloned()
}

pub(super) fn string(value: &Value, key: &str) -> Option<String> {
    match field(value, key) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    }
}

pub(super) fn list(value: &Value, key: &str) -> Option<Rc<RefCell<Vec<Value>>>> {
    match field(value, key) {
        Some(Value::List(items)) => Some(items),
        _ => None,
    }
}

pub(super) fn map(value: &Value) -> Option<Rc<RefCell<HashMap<String, Value>>>> {
    match value {
        Value::Map(map) => Some(map.clone()),
        _ => None,
    }
}
