//! Typed RGBA pixel decoding with WebGL unpack transforms.

use super::*;

const MAX_BYTES: usize = 16 * 1024 * 1024;

pub(super) enum Error {
    Invalid,
    OutOfMemory,
}

pub(super) fn decode(
    value: Option<&JsValue>,
    width: usize,
    height: usize,
    bindings: &texture_state::Bindings,
    allow_null: bool,
) -> Result<Vec<[u8; 4]>, Error> {
    let count = width.checked_mul(height).ok_or(Error::OutOfMemory)?;
    let bytes = count.checked_mul(4).ok_or(Error::OutOfMemory)?;
    if bytes > MAX_BYTES {
        return Err(Error::OutOfMemory);
    }
    if allow_null && matches!(value, None | Some(JsValue::Null)) {
        return Ok(vec![[0; 4]; count]);
    }
    let value = value.ok_or(Error::Invalid)?;
    let JsValue::Array(items) = value else {
        return Err(Error::Invalid);
    };
    let name = js::get_host_property(value, "__typed_array_name").map_err(|_| Error::Invalid)?;
    if !matches!(name, JsValue::String(name) if name == "Uint8Array" || name == "Uint8ClampedArray") {
        return Err(Error::Invalid);
    }
    let items = items.borrow();
    if items.len() != bytes {
        return Err(Error::Invalid);
    }
    Ok(pixel_transform::apply(&items, width, height, bindings))
}

pub(super) fn record(state: &mut WebGlState, error: Error) {
    let code = match error {
        Error::Invalid => webgl_constants::INVALID_VALUE,
        Error::OutOfMemory => webgl_constants::OUT_OF_MEMORY,
    };
    webgl_error::record(state, code);
}