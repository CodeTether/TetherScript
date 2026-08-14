//! Software document rendering for native windows.

use super::window::Window;

impl Window {
    pub(crate) fn render(&self, html: &str, css: &str) -> Result<Vec<u32>, String> {
        let image = crate::browser::render_document_to_raster(
            &crate::browser::parse_html(html),
            css,
            crate::browser::RenderOptions {
                viewport_width: self.width as i64,
                viewport_height: Some(self.height as i64),
                scale: 1,
                background: crate::browser::Rgba::WHITE,
            },
        )?;
        Ok(image.pixels.chunks_exact(4).map(rgb).collect())
    }
}

fn rgb(rgba: &[u8]) -> u32 {
    ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32
}
