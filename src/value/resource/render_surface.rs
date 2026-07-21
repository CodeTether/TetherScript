//! Owned quota-bound raster rendering surfaces.

use crate::browser::RasterImage;
use crate::value::Value;

use super::{render_surface_present, result};

pub(super) struct Handle {
    pub(super) width: i64,
    pub(super) height: usize,
    pub(super) scale: usize,
    pub(super) max_pixels: usize,
    pub(super) frame: Option<RasterImage>,
    pub(super) window: render_surface_present::Slot,
}

impl Handle {
    pub(super) fn call(&mut self, name: &str, values: &[Value]) -> Result<Value, String> {
        let value = match (name, values) {
            ("render", [html, css]) => result::value(self.render(html, css)),
            ("render_view", [view]) => result::value(self.render_view(view)),
            ("pixels", []) => result::value(self.pixels()),
            ("ppm", []) => result::value(self.ppm()),
            ("open_window", [title]) => self.open_window(title),
            ("present", []) => self.present(),
            ("close_window", []) => self.close_window(),
            ("clear", []) => result::nil(self.clear()),
            ("has_frame", []) => Value::Bool(self.frame.is_some()),
            ("is_window_open", []) => Value::Bool(self.is_window_open()),
            ("poll_input", []) => self.poll_input(),
            ("width", []) => Value::Int(self.frame_width() as i64),
            ("height", []) => Value::Int(self.frame_height() as i64),
            ("pixel_count", []) => Value::Int(self.pixel_count() as i64),
            ("capacity", []) => Value::Int(self.max_pixels as i64),
            _ => {
                return Err(format!(
                    "render_surface: no method `{name}` accepting {} arguments",
                    values.len()
                ))
            }
        };
        Ok(value)
    }
}
