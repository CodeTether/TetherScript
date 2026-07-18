use super::*;

#[path = "visual_viewport_events_dispatch_bindings.rs"]
mod bindings;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, registry: model::Registry) {
    bindings::install(object, registry);
}

pub(super) fn run(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    registry: &model::Registry,
    raw: JsValue,
    trusted: bool,
) -> Result<JsValue, String> {
    let event = super::super::event_object::create(object, raw, trusted);
    let event_type = super::super::event_object::event_type(&event);
    let this_value = JsValue::Object(object.clone());
    let entries = registry
        .borrow()
        .get(&event_type)
        .cloned()
        .unwrap_or_default();
    for entry in entries {
        if !listeners::contains(registry, &event_type, &entry.callback, entry.capture) {
            continue;
        }
        if entry.once {
            listeners::remove(registry, &event_type, &entry.callback, entry.capture);
        }
        js::call_function_with_this(
            entry.callback.clone(),
            this_value.clone(),
            std::slice::from_ref(&event),
        )?;
        if super::super::event_object::flag(&event, "__immediatePropagationStopped") {
            break;
        }
    }
    if !super::super::event_object::flag(&event, "__immediatePropagationStopped") {
        invoke::handler(object, &event_type, &event, this_value)?;
    }
    Ok(JsValue::Bool(!super::super::event_object::flag(
        &event,
        "defaultPrevented",
    )))
}
