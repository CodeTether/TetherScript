use super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, registry: model::Registry) {
    let for_remove = registry.clone();
    object.borrow_mut().insert(
        "removeEventListener".into(),
        native("visualViewport.removeEventListener", None, move |args| {
            let event_type = args.first().map(JsValue::display).unwrap_or_default();
            let callback = args.get(1).cloned().unwrap_or(JsValue::Undefined);
            let (capture, _) = options::parse(args.get(2));
            remove(&for_remove, &event_type, &callback, capture);
            Ok(JsValue::Undefined)
        }),
    );
}

pub(super) fn remove(
    registry: &model::Registry,
    event_type: &str,
    callback: &JsValue,
    capture: bool,
) {
    if let Some(entries) = registry.borrow_mut().get_mut(event_type) {
        entries.retain(|entry| entry.callback != *callback || entry.capture != capture);
    }
}

pub(super) fn contains(
    registry: &model::Registry,
    event_type: &str,
    callback: &JsValue,
    capture: bool,
) -> bool {
    registry.borrow().get(event_type).is_some_and(|entries| {
        entries
            .iter()
            .any(|entry| entry.callback == *callback && entry.capture == capture)
    })
}
