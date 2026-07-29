//! Repository task input and agent completion report.

use super::super::super::{model::Message, state::AgentState};
use super::State;
use eframe::egui::{self, Color32, RichText};

pub(super) fn show(ui: &mut egui::Ui, fix: &mut State, agent: &AgentState) -> bool {
    ui.heading("1. Describe the outcome");
    ui.label("The agent may inspect, edit, and test this workspace. It will not commit or push.");
    let editor = ui.add(
        egui::TextEdit::multiline(&mut fix.task)
            .hint_text(
                "Example: Fix the failing parser test, run focused tests, and report evidence.",
            )
            .desired_rows(8)
            .desired_width(f32::INFINITY),
    );
    let can_run = !agent.busy && !fix.task.trim().is_empty();
    let shortcut = can_run
        && editor.has_focus()
        && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl);
    let run = ui
        .add_enabled(can_run, egui::Button::new("Run agent fix  Ctrl+Enter / F5"))
        .clicked()
        || shortcut;
    ui.separator();
    ui.heading("2. Agent report");
    ui.label(RichText::new(&agent.status).color(Color32::from_rgb(120, 185, 255)));
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if let Some(message) = latest_report(&agent.messages) {
                ui.label(&message.text);
            } else {
                ui.label("No completed task yet.");
            }
        });
    run
}

fn latest_report(messages: &[Message]) -> Option<&Message> {
    messages
        .iter()
        .rev()
        .find(|message| message.role == "assistant")
}
