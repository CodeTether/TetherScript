use super::*;

#[path = "read_state.rs"]
mod state;

const MAX_READ_BYTES: usize = 1024 * 1024;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    listeners: events::ListenerList,
) {
    let for_text = object.clone();
    let text_listeners = listeners.clone();
    object.borrow_mut().insert(
        "readAsText".into(),
        native("FileReader.readAsText", Some(1), move |args| {
            read(&for_text, &text_listeners, args, false)
        }),
    );
    let for_data_url = object.clone();
    object.borrow_mut().insert(
        "readAsDataURL".into(),
        native("FileReader.readAsDataURL", Some(1), move |args| {
            read(&for_data_url, &listeners, args, true)
        }),
    );
}

fn read(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    listeners: &events::ListenerList,
    args: &[JsValue],
    data_url: bool,
) -> Result<JsValue, String> {
    let input = args.first().unwrap_or(&JsValue::Undefined);
    let data = blob::bytes(input).unwrap_or_else(|| input.display().into_bytes());
    state::set(object, "readyState", JsValue::Number(1.0));
    state::set(object, "error", JsValue::Null);
    events::dispatch(object, listeners, "loadstart")?;
    if data.len() > MAX_READ_BYTES {
        return state::fail(object, listeners, "NotReadableError");
    }
    let result = if data_url {
        state::data_url_value(input, &data)
    } else {
        JsValue::String(String::from_utf8_lossy(&data).into())
    };
    state::set(object, "result", result);
    state::set(object, "readyState", JsValue::Number(2.0));
    events::dispatch(object, listeners, "load")?;
    events::dispatch(object, listeners, "loadend")?;
    Ok(JsValue::Undefined)
}
