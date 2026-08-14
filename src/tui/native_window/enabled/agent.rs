//! Native agent dashboard using in-tree rendering and JSON-RPC.

#[path = "agent/client.rs"]
mod client;
#[path = "agent/input.rs"]
mod input;
#[path = "agent/model.rs"]
mod model;
#[path = "agent/protocol.rs"]
mod protocol;
#[path = "agent/state.rs"]
mod state;
#[path = "agent/view.rs"]
mod view;
#[path = "agent/view_style.rs"]
mod view_style;

use crate::value::Value;

pub(super) fn run(args: &[Value]) -> Result<Value, String> {
    model::require(&args[1], "vault")?;
    model::require(&args[2], "fs")?;
    let config = model::Config::parse(&args[0])?;
    let mut client = client::AgentClient::spawn(&config.agent_path)?;
    let mut state = state::AgentState::new(config.mode);
    let mut window = super::window::Window::open(&config.title, 1320, 820, "tui_native_agent")?;
    while window.is_open() {
        state.receive(&client);
        let text = window.take_text();
        if input::apply(&window.pressed(), &text, &mut state.input) {
            state.submit(&mut client)?;
        }
        let (html, css) = view::document(&state);
        let pixels = window.render(&html, &css)?;
        window.present(&pixels)?;
    }
    Ok(Value::Nil)
}
