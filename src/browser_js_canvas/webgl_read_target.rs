//! WebGL typed-array readback destinations.

use super::super::*;

pub(super) fn write(target: Option<&JsValue>, offset: usize, bytes: &[u8]) -> Result<(), String> {
    let Some(target) = target else {
        return Err("WebGLRenderingContext.readPixels: missing Uint8Array destination".into());
    };
    if !is_uint8_array(target) {
        return Err("WebGLRenderingContext.readPixels: expected Uint8Array destination".into());
    }
    let JsValue::Array(items) = target else {
        return Err("WebGLRenderingContext.readPixels: invalid Uint8Array storage".into());
    };
    let required = offset.checked_add(bytes.len()).ok_or_else(|| {
        "WebGLRenderingContext.readPixels: destination offset overflow".to_string()
    })?;
    let actual = items.borrow().len();
    if actual < required {
        return Err(format!(
            "WebGLRenderingContext.readPixels: destination needs {required} bytes, got {actual}"
        ));
    }
    let mut items = items.borrow_mut();
    for (index, byte) in bytes.iter().enumerate() {
        items[offset + index] = JsValue::Number(*byte as f64);
    }
    Ok(())
}

fn is_uint8_array(value: &JsValue) -> bool {
    matches!(
        js::get_host_property(value, "__typed_array"),
        Ok(JsValue::Bool(true))
    ) && matches!(
        js::get_host_property(value, "BYTES_PER_ELEMENT"),
        Ok(JsValue::Number(1.0))
    )
}
