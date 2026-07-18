use super::super::*;

#[path = "orientation_events_dispatch.rs"]
mod dispatch;
#[path = "orientation_events_listeners.rs"]
mod listeners;

pub(super) type Listeners = Rc<RefCell<Vec<JsValue>>>;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let listeners: Listeners = Rc::new(RefCell::new(Vec::new()));
    listeners::install(object, listeners.clone());
    dispatch::install(object, listeners);
}

pub(super) fn dispatch_change(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
) -> Result<(), String> {
    let dispatcher = object
        .borrow()
        .get("__tsDispatchOrientationChange")
        .cloned();
    if let Some(dispatcher) = dispatcher {
        js::call_function_with_this(
            dispatcher,
            JsValue::Object(object.clone()),
            &[JsValue::String("change".into())],
        )?;
    }
    Ok(())
}
