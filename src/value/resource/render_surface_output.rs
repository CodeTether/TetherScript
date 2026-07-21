//! Snapshot access for rendered frames.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::render_surface::Handle;

impl Handle {
    pub(super) fn pixels(&self) -> Result<Value, String> {
        let frame = self.frame.as_ref().ok_or_else(Self::missing_frame)?;
        Ok(Value::Bytes(Rc::new(RefCell::new(frame.pixels.clone()))))
    }

    pub(super) fn ppm(&self) -> Result<Value, String> {
        let frame = self.frame.as_ref().ok_or_else(Self::missing_frame)?;
        Ok(Value::Bytes(Rc::new(RefCell::new(frame.to_ppm()))))
    }

    pub(super) fn clear(&mut self) -> Result<(), String> {
        self.frame = None;
        Ok(())
    }

    pub(super) fn frame_width(&self) -> usize {
        self.frame.as_ref().map_or(0, |frame| frame.width)
    }

    pub(super) fn frame_height(&self) -> usize {
        self.frame.as_ref().map_or(0, |frame| frame.height)
    }

    pub(super) fn pixel_count(&self) -> usize {
        self.frame_width().saturating_mul(self.frame_height())
    }

    fn missing_frame() -> String {
        "render_surface: no frame has been rendered".into()
    }
}
