//! `ImageData` typed pixel storage and object shape.

use super::*;

pub(super) fn from_bytes(width: usize, height: usize, bytes: Vec<u8>) -> Result<JsValue, String> {
    let data = super::super::super::compat_host::typed_array::clamped::from_bytes(bytes)?;
    object(width, height, data)
}

pub(super) fn from_array(args: &[JsValue]) -> Result<JsValue, String> {
    let data = args.first().expect("checked array argument").clone();
    require_clamped(&data)?;
    let width = super::geometry::i64_value(args.get(1));
    let len = match &data {
        JsValue::Array(items) => items.borrow().len(),
        _ => 0,
    };
    let height = args
        .get(2)
        .map(|_| super::geometry::i64_value(args.get(2)))
        .unwrap_or_else(|| (len / (width.max(1) as usize * 4)) as i64);
    let (width, height) = super::size::dimensions(width, height, "ImageData")?;
    if len != width * height * 4 {
        return Err("ImageData: data length does not match width and height".into());
    }
    object(width, height, data)
}

fn object(width: usize, height: usize, data: JsValue) -> Result<JsValue, String> {
    let mut object = HashMap::new();
    object.insert("__image_data".into(), JsValue::Bool(true));
    object.insert("width".into(), JsValue::Number(width as f64));
    object.insert("height".into(), JsValue::Number(height as f64));
    object.insert("colorSpace".into(), JsValue::String("srgb".into()));
    object.insert("data".into(), data);
    let value = JsValue::Object(Rc::new(RefCell::new(object)));
    super::prototype::attach(&value)?;
    Ok(value)
}

fn require_clamped(data: &JsValue) -> Result<(), String> {
    match js::get_host_property(data, "__typed_array_name") {
        Ok(JsValue::String(name)) if name == "Uint8ClampedArray" => Ok(()),
        _ => Err("ImageData: expected Uint8ClampedArray data".into()),
    }
}
