//! WebGL `readPixels` size and allocation validation.

pub(super) fn validate(rect: [i64; 4]) -> Result<(), String> {
    let (width, height) = (rect[2], rect[3]);
    if width < 0 || height < 0 {
        return Err(format!(
            "WebGLRenderingContext.readPixels: negative size {width}x{height}"
        ));
    }
    let width = usize::try_from(width).map_err(|_| {
        format!("WebGLRenderingContext.readPixels: width {width} exceeds address space")
    })?;
    let height = usize::try_from(height).map_err(|_| {
        format!("WebGLRenderingContext.readPixels: height {height} exceeds address space")
    })?;
    let area = width.saturating_mul(height);
    if area > super::super::super::surface::MAX_PIXELS {
        return Err(format!(
            "WebGLRenderingContext.readPixels: {area} pixels exceeds limit"
        ));
    }
    Ok(())
}
