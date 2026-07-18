//! WebGL `readPixels` format validation.

use super::super::*;

pub(super) fn validate(format: Option<&JsValue>, kind: Option<&JsValue>) -> Result<(), String> {
    let format = super::super::webgl_values::u32_value(format);
    let kind = super::super::webgl_values::u32_value(kind);
    if format != super::super::webgl_constants::RGBA
        || kind != super::super::webgl_constants::UNSIGNED_BYTE
    {
        return Err(format!(
            "WebGLRenderingContext.readPixels: unsupported format 0x{format:04X} and type 0x{kind:04X}"
        ));
    }
    Ok(())
}
