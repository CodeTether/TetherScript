//! `ImageData` byte validation and RGBA decoding.

use super::*;

pub(super) fn decode(
    items: &[JsValue],
    width: usize,
    height: usize,
) -> Result<Vec<[u8; 4]>, String> {
    let expected = width.saturating_mul(height).saturating_mul(4);
    if items.len() != expected {
        return Err(format!(
            "CanvasRenderingContext2D.putImageData: source needs {expected} bytes, got {}",
            items.len()
        ));
    }
    let byte = super::super::super::compat_host::typed_array::clamped::byte;
    Ok(items
        .chunks_exact(4)
        .map(|pixel| {
            [
                byte(&pixel[0]),
                byte(&pixel[1]),
                byte(&pixel[2]),
                byte(&pixel[3]),
            ]
        })
        .collect())
}
