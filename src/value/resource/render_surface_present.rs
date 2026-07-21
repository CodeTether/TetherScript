//! Native-window lifecycle for rendered surface frames.

mod backend;
#[cfg(feature = "native-window")]
mod input;
#[cfg(feature = "native-window")]
mod pixels;

use crate::value::Value;

use super::{args, render_surface::Handle, result};
pub(crate) use backend::Slot;

impl Handle {
    pub(super) fn open_window(&mut self, title: &Value) -> Value {
        let width = self.width as usize * self.scale;
        let height = self.height * self.scale;
        let opened = args::string(title, "render_surface.open_window title")
            .and_then(|title| self.window.open(&title, width, height));
        result::nil(opened)
    }

    pub(super) fn present(&mut self) -> Value {
        let presented = self
            .frame
            .as_ref()
            .ok_or_else(|| "render_surface.present: no frame has been rendered".to_string())
            .and_then(|frame| self.window.present(frame));
        result::nil(presented)
    }

    pub(super) fn close_window(&mut self) -> Value {
        self.window.close();
        result::nil(Ok(()))
    }

    pub(super) fn is_window_open(&self) -> bool {
        self.window.is_open()
    }

    pub(super) fn poll_input(&self) -> Value {
        result::value(self.window.poll_input())
    }
}
