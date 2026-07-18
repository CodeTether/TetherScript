use super::*;

pub(super) fn handler(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    event_type: &str,
    event: &JsValue,
    this_value: JsValue,
) -> Result<(), String> {
    if let Some(handler) = object
        .borrow()
        .get(&format!("on{event_type}"))
        .cloned()
        .filter(callable)
    {
        js::call_function_with_this(handler, this_value, std::slice::from_ref(event))?;
    }
    Ok(())
}

fn callable(value: &JsValue) -> bool {
    matches!(
        value,
        JsValue::Function(_) | JsValue::BoundFunction(_) | JsValue::Native(_)
    )
}
