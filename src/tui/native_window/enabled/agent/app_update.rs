//! Native agent mode dispatch for each egui frame.

use eframe::egui;

use super::AgentApp;

pub(super) fn run(app: &mut AgentApp, context: &egui::Context) {
    if context.input(|input| input.key_pressed(egui::Key::Escape)) {
        context.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    super::super::actions::poll(app);
    if app.fix_mode {
        super::fix::update(app, context);
        return;
    }
    let Some(cpu) = &mut app.cpu else { return };
    if let Some(prompt) = super::super::layout::show(context, &mut app.state, cpu) {
        super::super::actions::submit(app, prompt);
    }
}
