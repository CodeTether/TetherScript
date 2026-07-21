//! HTML/CSS frame rendering for owned surfaces.

use crate::browser::{parse_html, render_document_to_raster, RenderOptions, Rgba};
use crate::value::Value;

use super::{args, render_surface::Handle};

impl Handle {
    pub(super) fn render(&mut self, html: &Value, css: &Value) -> Result<Value, String> {
        let html = args::string(html, "render_surface.render html")?;
        let css = args::string(css, "render_surface.render css")?;
        let image = render_document_to_raster(
            &parse_html(&html),
            &css,
            RenderOptions {
                viewport_width: self.width,
                viewport_height: Some(self.height as i64),
                scale: self.scale,
                background: Rgba::WHITE,
            },
        )?;
        let pixels = image.width.saturating_mul(image.height);
        if pixels > self.max_pixels {
            return Err(format!(
                "render_surface.render: backpressure: {pixels} pixels exceed capacity {}",
                self.max_pixels
            ));
        }
        self.frame = Some(image);
        Ok(Value::Int(pixels as i64))
    }
}
