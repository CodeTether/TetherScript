//! Canvas surface snapshots as `ImageData`.

use super::*;

pub(super) fn get(handle: &DomHandle, args: &[JsValue]) -> Result<JsValue, String> {
    let (x, y, width, height) = super::size::rectangle(args)?;
    let bytes = super::store::with_surface(handle, |surface| {
        let mut bytes = Vec::with_capacity(width * height * 4);
        for row in 0..height {
            for column in 0..width {
                bytes.extend(super::pixels::at(
                    surface,
                    x.saturating_add(column as i64),
                    y.saturating_add(row as i64),
                ));
            }
        }
        bytes
    });
    super::object::from_bytes(width, height, bytes)
}
