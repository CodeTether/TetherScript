use super::super::event;
use super::*;

#[path = "orientation_events_dispatch_bindings.rs"]
mod bindings;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, listeners: Listeners) {
    bindings::install(object, listeners);
}

pub(super) fn run(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    listeners: &Listeners,
    raw: JsValue,
    trusted: bool,
) -> Result<JsValue, String> {
    let event = event::create(object, raw, trusted);
    if event::event_type(&event) == "change" {
        let this_value = JsValue::Object(object.clone());
        for listener in listeners.borrow().clone() {
            js::call_function_with_this(
                listener,
                this_value.clone(),
                std::slice::from_ref(&event),
            )?;
        }
        if let Some(handler) = object.borrow().get("onchange").cloned().filter(callable) {
            js::call_function_with_this(handler, this_value, std::slice::from_ref(&event))?;
        }
    }
    Ok(JsValue::Bool(true))
}

fn callable(value: &JsValue) -> bool {
    matches!(
        value,
        JsValue::Function(_) | JsValue::BoundFunction(_) | JsValue::Native(_)
    )
}
