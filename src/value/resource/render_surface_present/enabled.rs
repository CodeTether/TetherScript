//! Minifb-backed native render window.

use crate::browser::RasterImage;

use super::super::{input, pixels};

#[derive(Default)]
pub(crate) struct Slot {
    window: Option<Box<minifb::Window>>,
    buffer: Vec<u32>,
}

impl Slot {
    pub(crate) fn open(&mut self, title: &str, width: usize, height: usize) -> Result<(), String> {
        if self.is_open() {
            return Err("render_surface.open_window: a window is already open".into());
        }
        self.window = Some(Box::new(
            minifb::Window::new(title, width, height, Default::default())
                .map_err(|error| format!("render_surface.open_window: {error}"))?,
        ));
        Ok(())
    }

    pub(crate) fn present(&mut self, image: &RasterImage) -> Result<(), String> {
        let window = self
            .window
            .as_mut()
            .filter(|window| window.is_open())
            .ok_or_else(|| "render_surface.present: no native window is open".to_string())?;
        self.buffer = pixels::convert(image)?;
        window
            .update_with_buffer(&self.buffer, image.width, image.height)
            .map_err(|error| format!("render_surface.present: {error}"))
    }

    pub(crate) fn close(&mut self) {
        self.window = None;
        self.buffer.clear();
    }

    pub(crate) fn poll_input(&self) -> Result<crate::value::Value, String> {
        self.window
            .as_ref()
            .filter(|window| window.is_open())
            .map(|window| input::snapshot(window))
            .ok_or_else(|| "render_surface.poll_input: no native window is open".into())
    }

    pub(crate) fn is_open(&self) -> bool {
        self.window.as_ref().is_some_and(|window| window.is_open())
    }
}
