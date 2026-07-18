//! Canvas image rectangle normalization and allocation limits.

use super::*;

pub(super) fn rectangle(args: &[JsValue]) -> Result<(i64, i64, usize, usize), String> {
    let x = super::geometry::i64_value(args.first());
    let y = super::geometry::i64_value(args.get(1));
    let width = super::geometry::i64_value(args.get(2));
    let height = super::geometry::i64_value(args.get(3));
    let (width, height) = dimensions(width, height, "getImageData")?;
    let x = if super::geometry::i64_value(args.get(2)) < 0 {
        x.saturating_sub(width as i64)
    } else {
        x
    };
    let y = if super::geometry::i64_value(args.get(3)) < 0 {
        y.saturating_sub(height as i64)
    } else {
        y
    };
    Ok((x, y, width, height))
}

pub(super) fn dimensions(width: i64, height: i64, method: &str) -> Result<(usize, usize), String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "CanvasRenderingContext2D.{method}: width and height must be non-zero"
        ));
    }
    let width = usize::try_from(width.unsigned_abs())
        .map_err(|_| format!("CanvasRenderingContext2D.{method}: width exceeds address space"))?;
    let height = usize::try_from(height.unsigned_abs())
        .map_err(|_| format!("CanvasRenderingContext2D.{method}: height exceeds address space"))?;
    let area = width.saturating_mul(height);
    if area > super::surface::MAX_PIXELS {
        return Err(format!(
            "CanvasRenderingContext2D.{method}: {area} pixels exceeds limit"
        ));
    }
    Ok((width, height))
}
