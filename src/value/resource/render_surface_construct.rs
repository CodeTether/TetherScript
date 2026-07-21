//! Construction and validation for rendering surfaces.

use super::{payload::Payload, render_surface::Handle, OwnedResource};

impl Handle {
    pub(super) fn new(
        width: usize,
        height: usize,
        scale: usize,
        max_pixels: usize,
    ) -> Result<Self, String> {
        let width = i64::try_from(width)
            .map_err(|_| "resource.render_surface: width is too large".to_string())?;
        if width == 0 || height == 0 || scale == 0 || max_pixels == 0 {
            return Err(
                "resource.render_surface: dimensions, scale, and capacity must be positive".into(),
            );
        }
        let pixels = (width as usize)
            .checked_mul(height)
            .and_then(|value| value.checked_mul(scale))
            .and_then(|value| value.checked_mul(scale))
            .ok_or_else(|| "resource.render_surface: pixel dimensions overflow".to_string())?;
        if pixels > max_pixels {
            return Err(format!(
                "resource.render_surface: backpressure: {pixels} pixels exceed capacity {max_pixels}"
            ));
        }
        Ok(Self {
            width,
            height,
            scale,
            max_pixels,
            frame: None,
        })
    }
}

impl OwnedResource {
    /// Create a fixed-size raster surface bounded by `max_pixels`.
    ///
    /// # Errors
    /// Returns an error for zero, overflowing, or over-capacity dimensions.
    ///
    /// # Examples
    /// ```
    /// use tetherscript::value::resource::OwnedResource;
    /// let surface = OwnedResource::render_surface(80, 25, 1, 2_000)?;
    /// assert!(!surface.is_closed());
    /// # Ok::<(), String>(())
    /// ```
    pub fn render_surface(
        width: usize,
        height: usize,
        scale: usize,
        max_pixels: usize,
    ) -> Result<Self, String> {
        Handle::new(width, height, scale, max_pixels)
            .map(|handle| Self::new(Payload::RenderSurface(handle)))
    }
}
