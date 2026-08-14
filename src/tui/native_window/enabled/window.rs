//! Framebuffer window backed by the in-tree document renderer.

#[path = "window_input.rs"]
mod input;
#[path = "window_present.rs"]
mod present;

use std::{cell::RefCell, rc::Rc};

use minifb::WindowOptions;

pub(super) struct Window {
    pub(super) inner: minifb::Window,
    pub(super) text: Rc<RefCell<Vec<char>>>,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) name: &'static str,
}

impl Window {
    pub(super) fn open(
        title: &str,
        width: usize,
        height: usize,
        name: &'static str,
    ) -> Result<Self, String> {
        let mut inner = minifb::Window::new(title, width, height, WindowOptions::default())
            .map_err(|error| format!("{name}: {error}"))?;
        let text = Rc::new(RefCell::new(Vec::new()));
        inner.set_input_callback(Box::new(input::Text::new(text.clone())));
        inner.set_target_fps(60);
        Ok(Self {
            inner,
            text,
            width,
            height,
            name,
        })
    }

    pub(super) fn show_document(&mut self, html: &str, css: &str) -> Result<(), String> {
        let pixels = self.render(html, css)?;
        while self.is_open() {
            self.present(&pixels)?;
        }
        Ok(())
    }
}
