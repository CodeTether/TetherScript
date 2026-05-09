use super::*;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    window.insert(
        "TextEncoder".into(),
        native("TextEncoder", Some(0), move |_| Ok(encoder_object())),
    );
    window.insert(
        "TextDecoder".into(),
        native("TextDecoder", None, move |_| Ok(decoder_object())),
    );
}

fn encoder_object() -> JsValue {
    let mut obj = HashMap::from([("encoding".into(), JsValue::String("utf-8".into()))]);
    obj.insert(
        "encode".into(),
        native("TextEncoder.encode", None, move |args| {
            let text = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(bytes::byte_array(text.into_bytes()))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn decoder_object() -> JsValue {
    let mut obj = HashMap::from([("encoding".into(), JsValue::String("utf-8".into()))]);
    obj.insert(
        "decode".into(),
        native("TextDecoder.decode", None, move |args| {
            let input = args.first().unwrap_or(&JsValue::Undefined);
            let bytes = bytes::bytes_from_value(input);
            Ok(JsValue::String(String::from_utf8_lossy(&bytes).into()))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}
