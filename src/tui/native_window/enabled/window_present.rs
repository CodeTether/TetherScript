//! Native framebuffer presentation and input access.

use minifb::{Key, KeyRepeat};

use super::Window;

impl Window {
    pub(crate) fn present(&mut self, pixels: &[u32]) -> Result<(), String> {
        self.inner
            .update_with_buffer(pixels, self.width, self.height)
            .map_err(|error| format!("{}: {error}", self.name))
    }

    pub(crate) fn is_open(&self) -> bool {
        self.inner.is_open() && !self.inner.is_key_down(Key::Escape)
    }

    pub(crate) fn pressed(&self) -> Vec<Key> {
        self.inner.get_keys_pressed(KeyRepeat::Yes)
    }

    pub(crate) fn take_text(&self) -> Vec<char> {
        self.text.borrow_mut().drain(..).collect()
    }
}
