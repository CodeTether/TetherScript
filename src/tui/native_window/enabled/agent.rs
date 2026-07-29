//! Native agent window backed by the existing agent TUI RPC host.

#[path = "agent/actions.rs"]
mod actions;
#[path = "agent/app.rs"]
mod app;
#[path = "agent/chat.rs"]
mod chat;
#[path = "agent/client.rs"]
mod client;
#[path = "agent/composer.rs"]
mod composer;
#[path = "agent/disassembly.rs"]
mod disassembly;
#[path = "agent/disassembly_decode.rs"]
mod disassembly_decode;
#[path = "agent/disassembly_view.rs"]
mod disassembly_view;
#[path = "agent/layout.rs"]
mod layout;
#[path = "agent/model.rs"]
mod model;
#[path = "agent/protocol.rs"]
mod protocol;
#[path = "agent/state.rs"]
mod state;
#[cfg(test)]
#[path = "agent/tests.rs"]
mod tests;

use crate::value::Value;

pub(super) fn run(args: &[Value]) -> Result<Value, String> {
    model::require(&args[1], "vault")?;
    model::require(&args[2], "fs")?;
    let config = model::Config::parse(&args[0])?;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 820.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        &config.title.clone(),
        options,
        Box::new(move |context| {
            super::theme::install(&context.egui_ctx);
            Box::new(app::AgentApp::new(config, context.egui_ctx.clone()))
        }),
    )
    .map_err(|error| format!("tui_native_agent: {error}"))?;
    Ok(Value::Nil)
}
