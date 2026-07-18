use super::super::*;

#[path = "visual_viewport_events_dispatch.rs"]
mod dispatch;
#[path = "visual_viewport_events_invoke.rs"]
mod invoke;
#[path = "visual_viewport_events_listener_add.rs"]
mod listener_add;
#[path = "visual_viewport_events_listener_remove.rs"]
mod listener_remove;
#[path = "visual_viewport_events_listeners.rs"]
mod listeners;
#[path = "visual_viewport_events_model.rs"]
mod model;
#[path = "visual_viewport_events_options.rs"]
mod options;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let registry = model::new_registry();
    object.borrow_mut().insert("onresize".into(), JsValue::Null);
    object.borrow_mut().insert("onscroll".into(), JsValue::Null);
    object
        .borrow_mut()
        .insert("onscrollend".into(), JsValue::Null);
    listeners::install(object, registry.clone());
    dispatch::install(object, registry);
}

pub(super) fn dispatch_type(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    event_type: &str,
) -> Result<(), String> {
    let dispatcher = object.borrow().get("__tsDispatchViewportEvent").cloned();
    if let Some(dispatcher) = dispatcher {
        js::call_function_with_this(
            dispatcher,
            JsValue::Object(object.clone()),
            &[JsValue::String(event_type.into())],
        )?;
    }
    Ok(())
}
