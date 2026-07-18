//! Trusted events dispatched by native window operations.

use super::*;

pub(super) fn dispatch(window: &JsValue, event_type: &str) -> Result<(), String> {
    let event = JsValue::Object(Rc::new(RefCell::new(HashMap::from([(
        "isTrusted".into(),
        JsValue::Bool(true),
    )]))));
    let event = normalize_event(event, event_type, window.clone(), window.clone());
    dispatch_window_normalized(event_type, event, window.clone())
}
