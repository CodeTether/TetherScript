//! In-tree native TUI rendered through the software browser rasterizer.

#[path = "enabled/agent.rs"]
mod agent;
#[path = "enabled/window.rs"]
mod window;
#[path = "enabled/window_render.rs"]
mod window_render;

use crate::value::Value;

pub(crate) fn builtin(args: &[Value]) -> Result<Value, String> {
    let view = super::super::view::parse(&args[0])?;
    let title = view.title.clone();
    let (html, css) = crate::interp::tui::native_document(&args[0])?;
    let mut window = window::Window::open(&title, 1040, 720, "tui_native")?;
    window.show_document(&html, &css)?;
    Ok(Value::Nil)
}

pub(crate) fn agent_builtin(args: &[Value]) -> Result<Value, String> {
    agent::run(args)
}
