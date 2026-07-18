//! Raw byte extraction from JavaScript buffer sources.

use super::*;

pub(super) const MAX_BYTES: usize = 16 * 1024 * 1024;

pub(super) enum Error {
    Invalid,
    OutOfMemory,
}

pub(super) fn bytes(value: Option<&JsValue>) -> Result<Vec<u8>, Error> {
    let value = value.ok_or(Error::Invalid)?;
    match value {
        JsValue::Number(size) if size.is_finite() && *size >= 0.0 => allocate(*size as usize),
        JsValue::Object(object) if is_array_buffer(&object.borrow()) => {
            allocate(length(&object.borrow()))
        }
        JsValue::Array(items) => typed_bytes::decode(value, &items.borrow()),
        _ => Err(Error::Invalid),
    }
}

pub(super) fn data(value: Option<&JsValue>) -> Result<Vec<u8>, Error> {
    if matches!(value, Some(JsValue::Number(_))) {
        return Err(Error::Invalid);
    }
    bytes(value)
}

fn allocate(len: usize) -> Result<Vec<u8>, Error> {
    if len > MAX_BYTES {
        Err(Error::OutOfMemory)
    } else {
        Ok(vec![0; len])
    }
}

fn is_array_buffer(object: &HashMap<String, JsValue>) -> bool {
    matches!(object.get("__array_buffer"), Some(JsValue::Bool(true)))
}

fn length(object: &HashMap<String, JsValue>) -> usize {
    match object.get("byteLength") {
        Some(JsValue::Number(value)) if value.is_finite() => *value as usize,
        _ => 0,
    }
}
