//! Headless backend used when native windows are not enabled.

use crate::browser::RasterImage;

#[derive(Default)]
pub(crate) struct Slot;

impl Slot {
    pub(crate) fn open(
        &mut self,
        _title: &str,
        _width: usize,
        _height: usize,
    ) -> Result<(), String> {
        Err("render_surface.open_window: enable the `native-window` feature".into())
    }

    pub(crate) fn present(&mut self, _image: &RasterImage) -> Result<(), String> {
        Err("render_surface.present: no native window is open".into())
    }

    pub(crate) fn close(&mut self) {}

    pub(crate) fn poll_input(&self) -> Result<crate::value::Value, String> {
        Err("render_surface.poll_input: no native window is open".into())
    }

    pub(crate) fn is_open(&self) -> bool {
        false
    }
}
