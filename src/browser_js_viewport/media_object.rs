use super::*;

pub(super) type Listeners = Rc<RefCell<Vec<JsValue>>>;

pub(super) fn create(media: String, queries: Rc<RefCell<Vec<MediaQueryHandle>>>) -> JsValue {
    let object = Rc::new(RefCell::new(HashMap::new()));
    let listeners = Rc::new(RefCell::new(Vec::new()));
    {
        let mut map = object.borrow_mut();
        map.insert(
            "matches".into(),
            JsValue::Bool(media_query::matches(
                &media,
                constants::DEFAULT_VIEWPORT_WIDTH,
            )),
        );
        map.insert("media".into(), JsValue::String(media));
        map.insert("onchange".into(), JsValue::Null);
        map.insert(
            "__media_listeners".into(),
            JsValue::Array(listeners.clone()),
        );
        media_legacy::install(&mut map, listeners.clone());
        media_event::install(&mut map, object.clone(), listeners);
    }
    queries.borrow_mut().push(MediaQueryHandle {
        query: media,
        object: object.clone(),
    });
    JsValue::Object(object)
}

pub(super) fn update(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    media: &str,
    width: i64,
) -> Result<bool, String> {
    let next = media_query::matches(media, width);
    let old = object
        .borrow()
        .get("matches")
        .is_some_and(|value| matches!(value, JsValue::Bool(true)));
    object
        .borrow_mut()
        .insert("matches".into(), JsValue::Bool(next));
    Ok(old != next)
}

pub(super) fn dispatch_change(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
) -> Result<(), String> {
    let listeners = object
        .borrow()
        .get("__media_listeners")
        .cloned()
        .and_then(listeners_from_value)
        .unwrap_or_default();
    media_dispatch::dispatch(
        object.clone(),
        Rc::new(RefCell::new(listeners)),
        JsValue::String("change".into()),
    )?;
    Ok(())
}

fn listeners_from_value(value: JsValue) -> Option<Vec<JsValue>> {
    let JsValue::Array(items) = value else {
        return None;
    };
    Some(items.borrow().clone())
}
