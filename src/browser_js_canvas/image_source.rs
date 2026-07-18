//! Validated `ImageData` pixel sources.

use super::*;

pub(super) struct Source {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>,
}

pub(super) fn parse(value: Option<&JsValue>) -> Result<Source, String> {
    let value = value.ok_or_else(|| {
        "CanvasRenderingContext2D.putImageData: missing ImageData source".to_string()
    })?;
    if !matches!(
        js::get_host_property(value, "__image_data"),
        Ok(JsValue::Bool(true))
    ) {
        return Err("CanvasRenderingContext2D.putImageData: expected ImageData source".into());
    }
    let width = super::source_dimension::get(value, "width")?;
    let height = super::source_dimension::get(value, "height")?;
    let data = js::get_host_property(value, "data")
        .map_err(|_| "CanvasRenderingContext2D.putImageData: source has no data".to_string())?;
    if !matches!(
        js::get_host_property(&data, "__typed_array_name"),
        Ok(JsValue::String(name)) if name == "Uint8ClampedArray"
    ) {
        return Err(
            "CanvasRenderingContext2D.putImageData: source data must be Uint8ClampedArray".into(),
        );
    }
    let JsValue::Array(items) = data else {
        return Err(
            "CanvasRenderingContext2D.putImageData: source data is not typed pixels".into(),
        );
    };
    let pixels = super::source_pixels::decode(&items.borrow(), width, height)?;
    Ok(Source {
        width,
        height,
        pixels,
    })
}
