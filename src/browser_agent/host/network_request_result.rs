//! Validation for normalized agent network responses.

use crate::value::Value;

pub(super) fn validate(value: Value, action: &str) -> Result<Value, String> {
    let Value::Map(map) = &value else {
        return Err(format!(
            "browser.{action}: request did not settle to a response"
        ));
    };
    let error = match map.borrow().get("error") {
        Some(Value::Str(error)) => Some((**error).clone()),
        _ => None,
    };
    match error {
        Some(error) => Err(format!("browser.{action}: {error}")),
        None => Ok(value),
    }
}
