//! Optional action-envelope value extraction.

use crate::value::Value;

pub(super) fn int(payload: &Value, name: &str) -> Result<Option<i64>, String> {
    let Value::Map(map) = payload else {
        return Err("browser host: action payload must be map".into());
    };
    match map.borrow().get(name) {
        Some(Value::Int(value)) => Ok(Some(*value)),
        Some(value) => Err(format!(
            "browser host: `{}` must be int, got {}",
            name,
            value.type_name()
        )),
        None => Ok(None),
    }
}

pub(super) fn string(payload: &Value, name: &str) -> Result<Option<String>, String> {
    let Value::Map(map) = payload else {
        return Err("browser host: action payload must be map".into());
    };
    match map.borrow().get(name) {
        Some(Value::Str(value)) => Ok(Some((**value).clone())),
        Some(value) => Err(format!(
            "browser host: `{}` must be str, got {}",
            name,
            value.type_name()
        )),
        None => Ok(None),
    }
}

pub(super) fn bool(payload: &Value, name: &str) -> Result<Option<bool>, String> {
    let Value::Map(map) = payload else {
        return Err("browser host: action payload must be map".into());
    };
    match map.borrow().get(name) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(value) => Err(format!(
            "browser host: `{}` must be bool, got {}",
            name,
            value.type_name()
        )),
        None => Ok(None),
    }
}
