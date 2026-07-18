use super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, registry: model::Registry) {
    object.borrow_mut().insert(
        "addEventListener".into(),
        native("visualViewport.addEventListener", None, move |args| {
            let event_type = args.first().map(JsValue::display).unwrap_or_default();
            let callback = args.get(1).cloned().unwrap_or(JsValue::Undefined);
            let (capture, once) = options::parse(args.get(2));
            if event_type.is_empty() || !callable(&callback) {
                return Ok(JsValue::Undefined);
            }
            let mut registry = registry.borrow_mut();
            let entries = registry.entry(event_type).or_default();
            let duplicate = entries
                .iter()
                .any(|entry| entry.callback == callback && entry.capture == capture);
            if !duplicate {
                entries.push(model::Listener {
                    callback,
                    capture,
                    once,
                });
            }
            Ok(JsValue::Undefined)
        }),
    );
}

fn callable(value: &JsValue) -> bool {
    matches!(
        value,
        JsValue::Function(_) | JsValue::BoundFunction(_) | JsValue::Native(_)
    )
}
