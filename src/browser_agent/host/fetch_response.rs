//! HTTP response extraction for native document navigation.

use crate::value::Value;

pub(super) struct ResponseParts {
    pub(super) status: i64,
    pub(super) body: String,
    pub(super) location: Option<String>,
}

pub(super) fn parts(response: Value) -> Result<ResponseParts, String> {
    let Value::Map(response) = response else {
        return Err("browser.goto: HTTP response must be map".into());
    };
    let response = response.borrow();
    let status = match response.get("status") {
        Some(Value::Int(value)) => *value,
        _ => return Err("browser.goto: HTTP response has no status".into()),
    };
    let body = match response.get("body") {
        Some(Value::Str(value)) => (**value).clone(),
        _ => return Err("browser.goto: HTTP response has no body".into()),
    };
    Ok(ResponseParts {
        status,
        body,
        location: response.get("headers").and_then(location_header),
    })
}

fn location_header(headers: &Value) -> Option<String> {
    let Value::Map(headers) = headers else {
        return None;
    };
    match headers.borrow().get("location") {
        Some(Value::Str(value)) => Some((**value).clone()),
        _ => None,
    }
}
