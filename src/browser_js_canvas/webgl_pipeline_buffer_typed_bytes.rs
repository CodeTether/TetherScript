//! Typed-array element serialization into raw little-endian bytes.

use super::*;

pub(super) fn decode(value: &JsValue, items: &[JsValue]) -> Result<Vec<u8>, source::Error> {
    let JsValue::String(name) =
        js::get_host_property(value, "__typed_array_name").map_err(|_| source::Error::Invalid)?
    else {
        return Err(source::Error::Invalid);
    };
    let width = element_width(&name).ok_or(source::Error::Invalid)?;
    let len = items
        .len()
        .checked_mul(width)
        .ok_or(source::Error::OutOfMemory)?;
    if len > source::MAX_BYTES {
        return Err(source::Error::OutOfMemory);
    }
    let mut bytes = Vec::with_capacity(len);
    for item in items {
        let number = item.display().parse::<f64>().unwrap_or(0.0);
        match name.as_str() {
            "Uint8Array" | "Uint8ClampedArray" => bytes.push(number as u8),
            "Uint16Array" => bytes.extend((number as u16).to_le_bytes()),
            "Uint32Array" => bytes.extend((number as u32).to_le_bytes()),
            "Int32Array" => bytes.extend((number as i32).to_le_bytes()),
            "Float32Array" => bytes.extend((number as f32).to_le_bytes()),
            _ => return Err(source::Error::Invalid),
        }
    }
    Ok(bytes)
}

fn element_width(name: &str) -> Option<usize> {
    match name {
        "Uint8Array" | "Uint8ClampedArray" => Some(1),
        "Uint16Array" => Some(2),
        "Uint32Array" | "Int32Array" | "Float32Array" => Some(4),
        _ => None,
    }
}
