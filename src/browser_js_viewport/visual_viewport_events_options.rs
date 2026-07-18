use super::*;

pub(super) fn parse(value: Option<&JsValue>) -> (bool, bool) {
    match value {
        Some(JsValue::Bool(capture)) => (*capture, false),
        Some(JsValue::Object(object)) => {
            let object = object.borrow();
            (
                object.get("capture").is_some_and(JsValue::truthy),
                object.get("once").is_some_and(JsValue::truthy),
            )
        }
        _ => (false, false),
    }
}
