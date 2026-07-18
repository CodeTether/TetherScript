use super::*;

#[path = "visual_viewport_event_methods.rs"]
mod event_methods;
#[path = "visual_viewport_event_object.rs"]
mod event_object;
#[path = "visual_viewport_events.rs"]
mod events;
#[path = "visual_viewport_metrics.rs"]
mod metrics;
#[cfg(test)]
#[path = "tests_visual_viewport_events.rs"]
mod tests_visual_viewport_events;
#[cfg(test)]
#[path = "tests_visual_viewport_listener_mutation.rs"]
mod tests_visual_viewport_listener_mutation;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    let object = Rc::new(RefCell::new(HashMap::new()));
    metrics::install(&mut object.borrow_mut());
    events::install(&object);
    window.insert("visualViewport".into(), JsValue::Object(object));
}

pub(in crate::browser_js) fn sync(window: &JsValue) {
    if let Some((window, viewport)) = objects(window) {
        metrics::sync(&window, &viewport);
    }
}

pub(in crate::browser_js) fn dispatch(window: &JsValue, event_type: &str) -> Result<(), String> {
    let Some((_, viewport)) = objects(window) else {
        return Ok(());
    };
    events::dispatch_type(&viewport, event_type)
}

fn objects(window: &JsValue) -> Option<(Object, Object)> {
    let JsValue::Object(window) = window else {
        return None;
    };
    let JsValue::Object(viewport) = window.borrow().get("visualViewport")?.clone() else {
        return None;
    };
    Some((window.clone(), viewport))
}

type Object = Rc<RefCell<HashMap<String, JsValue>>>;
