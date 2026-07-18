//! Standards-shaped `ImageData` object construction.

use super::*;

pub(super) fn construct(args: &[JsValue]) -> Result<JsValue, String> {
    if matches!(args.first(), Some(JsValue::Array(_))) {
        return super::object::from_array(args);
    }
    blank_dimensions(args, "ImageData")
}

pub(super) fn blank(args: &[JsValue]) -> Result<JsValue, String> {
    if matches!(args.first(), Some(JsValue::Object(_))) {
        let source = args.first().expect("checked ImageData argument");
        if !matches!(
            js::get_host_property(source, "__image_data"),
            Ok(JsValue::Bool(true))
        ) {
            return Err(
                "CanvasRenderingContext2D.createImageData: expected ImageData source".into(),
            );
        }
        let width = property_i64(source, "width")?;
        let height = property_i64(source, "height")?;
        return from_blank_size(width, height, "createImageData");
    }
    blank_dimensions(args, "createImageData")
}

fn blank_dimensions(args: &[JsValue], method: &str) -> Result<JsValue, String> {
    let width = super::geometry::i64_value(args.first());
    let height = super::geometry::i64_value(args.get(1));
    from_blank_size(width, height, method)
}

fn from_blank_size(width: i64, height: i64, method: &str) -> Result<JsValue, String> {
    let (width, height) = super::size::dimensions(width, height, method)?;
    super::object::from_bytes(width, height, vec![0; width * height * 4])
}

fn property_i64(value: &JsValue, name: &str) -> Result<i64, String> {
    let value = js::get_host_property(value, name)
        .map_err(|_| format!("createImageData: source has no {name}"))?;
    Ok(super::geometry::i64_value(Some(&value)))
}
