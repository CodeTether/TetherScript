use super::*;

pub(super) fn fail(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    listeners: &events::ListenerList,
    message: &str,
) -> Result<JsValue, String> {
    set(object, "result", JsValue::Null);
    set(object, "error", JsValue::String(message.into()));
    set(object, "readyState", JsValue::Number(2.0));
    events::dispatch(object, listeners, "error")?;
    events::dispatch(object, listeners, "loadend")?;
    Ok(JsValue::Undefined)
}

pub(super) fn data_url_value(input: &JsValue, data: &[u8]) -> JsValue {
    let mime = match blob::mime_type(input).as_str() {
        "" => "application/octet-stream".into(),
        value => value.to_string(),
    };
    JsValue::String(format!("data:{mime};base64,{}", base64_encode(data)))
}

pub(super) fn set(object: &Rc<RefCell<HashMap<String, JsValue>>>, name: &str, value: JsValue) {
    object.borrow_mut().insert(name.into(), value);
}
