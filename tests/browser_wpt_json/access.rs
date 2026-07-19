use std::collections::HashMap;

use tetherscript::value::Value;

pub fn object<'a>(
    map: &'a HashMap<String, Value>,
    key: &str,
) -> Result<&'a std::rc::Rc<std::cell::RefCell<HashMap<String, Value>>>, String> {
    match map.get(key) {
        Some(Value::Map(value)) => Ok(value),
        _ => Err(format!("fixture field `{key}` must be an object")),
    }
}

pub fn string(map: &HashMap<String, Value>, key: &str) -> Result<String, String> {
    match map.get(key) {
        Some(Value::Str(value)) if !value.is_empty() => Ok(value.as_ref().clone()),
        _ => Err(format!("fixture field `{key}` must be a non-empty string")),
    }
}

pub fn strings(map: &HashMap<String, Value>, key: &str) -> Result<Vec<String>, String> {
    let Some(Value::List(values)) = map.get(key) else {
        return Err(format!("fixture field `{key}` must be an array"));
    };
    values
        .borrow()
        .iter()
        .map(|value| match value {
            Value::Str(value) if !value.is_empty() => Ok(value.as_ref().clone()),
            _ => Err(format!(
                "fixture field `{key}` must contain non-empty strings"
            )),
        })
        .collect()
}
