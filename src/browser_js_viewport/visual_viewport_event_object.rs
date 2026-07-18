use super::super::*;

pub(super) fn create(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    event: JsValue,
    trusted: bool,
) -> JsValue {
    let event = match event {
        JsValue::Object(event) => event,
        JsValue::String(event_type) => Rc::new(RefCell::new(HashMap::from([(
            "type".into(),
            JsValue::String(event_type),
        )]))),
        _ => Rc::new(RefCell::new(HashMap::new())),
    };
    {
        let mut map = event.borrow_mut();
        map.entry("type".into())
            .or_insert_with(|| JsValue::String("event".into()));
        map.insert("target".into(), JsValue::Object(object.clone()));
        map.insert("currentTarget".into(), JsValue::Object(object.clone()));
        map.insert("isTrusted".into(), JsValue::Bool(trusted));
        map.entry("bubbles".into()).or_insert(JsValue::Bool(false));
        map.entry("cancelable".into())
            .or_insert(JsValue::Bool(false));
        map.entry("defaultPrevented".into())
            .or_insert(JsValue::Bool(false));
    }
    super::event_methods::install(&event);
    JsValue::Object(event)
}

pub(super) fn event_type(event: &JsValue) -> String {
    match event {
        JsValue::Object(event) => event
            .borrow()
            .get("type")
            .map(JsValue::display)
            .unwrap_or_else(|| "event".into()),
        _ => "event".into(),
    }
}

pub(super) fn flag(event: &JsValue, name: &str) -> bool {
    matches!(event, JsValue::Object(object) if object.borrow().get(name).is_some_and(JsValue::truthy))
}
