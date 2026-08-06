//! Small helpers for reading typed fields from TetherScript maps.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) type ValueMap = Rc<RefCell<HashMap<String, Value>>>;

pub(super) fn new_map() -> ValueMap {
    Rc::new(RefCell::new(HashMap::new()))
}

pub(super) fn set(map: &ValueMap, key: &str, value: Value) {
    map.borrow_mut().insert(key.into(), value);
}

pub(super) fn get_str(map: &HashMap<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::Str(s)) => (**s).clone(),
        _ => String::new(),
    }
}

pub(super) fn get_bool(map: &HashMap<String, Value>, key: &str) -> bool {
    matches!(map.get(key), Some(Value::Bool(true)))
}

pub(super) fn get_opt_bool(map: &HashMap<String, Value>, key: &str) -> Option<bool> {
    match map.get(key) {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    }
}

pub(super) fn get_list(map: &HashMap<String, Value>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(Value::List(items)) => items
            .borrow()
            .iter()
            .filter_map(|v| match v {
                Value::Str(s) => Some((**s).clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn list_of(items: &[String]) -> Value {
    let list: Vec<Value> = items
        .iter()
        .map(|s| Value::Str(Rc::new(s.clone())))
        .collect();
    Value::List(Rc::new(RefCell::new(list)))
}
