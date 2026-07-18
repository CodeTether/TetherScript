//! `ImageData` source dimension validation.

use super::*;

pub(super) fn get(value: &JsValue, name: &str) -> Result<usize, String> {
    let value = js::get_host_property(value, name)
        .map_err(|_| format!("CanvasRenderingContext2D.putImageData: source has no {name}"))?;
    let JsValue::Number(number) = value else {
        return Err(format!(
            "CanvasRenderingContext2D.putImageData: source {name} is not a number"
        ));
    };
    if !number.is_finite() || number <= 0.0 {
        return Err(format!(
            "CanvasRenderingContext2D.putImageData: invalid source {name} {number}"
        ));
    }
    Ok(number as usize)
}
