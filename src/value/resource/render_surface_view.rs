//! Shared TUI-view rendering for native surfaces.

use std::rc::Rc;

use crate::value::Value;

use super::render_surface::Handle;

impl Handle {
    pub(super) fn render_view(&mut self, view: &Value) -> Result<Value, String> {
        let (html, css) = crate::interp::tui::native_document(view)?;
        self.render(&Value::Str(Rc::new(html)), &Value::Str(Rc::new(css)))
    }
}
