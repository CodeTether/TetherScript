//! Egui-backed native TUI window.

#[path = "enabled/agent.rs"]
mod agent;
#[path = "enabled/app.rs"]
mod app;
#[path = "enabled/footer.rs"]
mod footer;
#[path = "enabled/header.rs"]
mod header;
#[path = "enabled/model.rs"]
mod model;
#[path = "enabled/panels.rs"]
mod panels;
#[path = "enabled/sidebar.rs"]
mod sidebar;
#[path = "enabled/tasks.rs"]
mod tasks;
#[path = "enabled/theme.rs"]
mod theme;

use crate::value::Value;

pub(crate) fn builtin(args: &[Value]) -> Result<Value, String> {
    let view = super::super::view::parse(&args[0])?;
    let dashboard = model::Dashboard::from(view);
    let title = dashboard.title.clone();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |context| {
            theme::install(&context.egui_ctx);
            Box::new(app::DashboardApp::new(dashboard))
        }),
    )
    .map_err(|error| format!("tui_native: {error}"))?;
    Ok(Value::Nil)
}

pub(crate) fn agent_builtin(args: &[Value]) -> Result<Value, String> {
    agent::run(args)
}
