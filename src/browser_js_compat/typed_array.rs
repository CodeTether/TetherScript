use super::*;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    window.insert(
        "Uint8Array".into(),
        native("Uint8Array", None, move |args| {
            let source = args.first().unwrap_or(&JsValue::Number(0.0));
            Ok(uint8_array(source))
        }),
    );
}

fn uint8_array(source: &JsValue) -> JsValue {
    match source {
        JsValue::Number(len) if len.is_finite() && *len > 0.0 => {
            bytes::byte_array(std::iter::repeat(0).take(*len as usize))
        }
        JsValue::Array(_) | JsValue::Object(_) | JsValue::String(_) => {
            bytes::byte_array(bytes::bytes_from_value(source))
        }
        _ => bytes::byte_array([]),
    }
}
