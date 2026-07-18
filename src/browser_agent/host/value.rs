//! Runtime value helpers for browser host action envelopes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) use super::value_optional::{
    bool as optional_bool, int as optional_int, string as optional_string,
};

pub(super) fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(Rc::new(RefCell::new(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<HashMap<_, _>>(),
    )))
}

pub(super) fn string(value: impl Into<String>) -> Value {
    Value::Str(Rc::new(value.into()))
}

pub(super) fn list(values: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(values)))
}

pub(super) fn field(payload: &Value, name: &str) -> Result<Value, String> {
    let Value::Map(map) = payload else {
        return Err("browser host: action payload must be map".into());
    };
    map.borrow()
        .get(name)
        .cloned()
        .ok_or_else(|| format!("browser host: action missing `{}`", name))
}

pub(super) fn string_field(payload: &Value, name: &str) -> Result<String, String> {
    match field(payload, name)? {
        Value::Str(value) => Ok((*value).clone()),
        value => Err(format!(
            "browser host: `{}` must be str, got {}",
            name,
            value.type_name()
        )),
    }
}
