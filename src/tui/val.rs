//! Value conversion helpers for terminal built-ins.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(super) fn strv(text: impl Into<String>) -> Value {
    Value::Str(Rc::new(text.into()))
}

pub(super) fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(value) => Ok(*value),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}

pub(super) fn bool_arg(value: &Value, label: &str) -> Result<bool, String> {
    match value {
        Value::Bool(value) => Ok(*value),
        other => Err(format!("{label} must be bool, got {}", other.type_name())),
    }
}

pub(super) fn map_arg<'a>(
    value: &'a Value,
    label: &str,
) -> Result<std::cell::Ref<'a, HashMap<String, Value>>, String> {
    match value {
        Value::Map(value) => Ok(value.borrow()),
        other => Err(format!("{label} must be map, got {}", other.type_name())),
    }
}

pub(super) fn map_value(items: impl IntoIterator<Item = (String, Value)>) -> Value {
    Value::Map(Rc::new(RefCell::new(items.into_iter().collect())))
}

pub(super) fn result(value: Result<Value, String>) -> Value {
    Value::Result(Rc::new(match value {
        Ok(value) => ResultValue::Ok(value),
        Err(error) => ResultValue::Err(error),
    }))
}
