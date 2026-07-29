//! Native agent shell layout and CPU inspector composition.

#[path = "agent_header.rs"]
mod header;

use eframe::egui::{self, Color32, RichText};

use super::{chat, composer, disassembly::Snapshot, disassembly_view, state::AgentState};

pub(super) fn show(
    context: &egui::Context,
    state: &mut AgentState,
    cpu: &mut Snapshot,
) -> Option<String> {
    header::show(context, state);
    egui::SidePanel::right("cpu_state")
        .default_width(610.0)
        .resizable(true)
        .show(context, |ui| {
            ui.add_space(14.0);
            disassembly_view::show(ui, cpu);
        });
    let mut prompt = None;
    egui::TopBottomPanel::bottom("agent_composer").show(context, |ui| {
        ui.add_space(8.0);
        prompt = composer::show(ui, &mut state.input, state.busy);
        ui.add_space(8.0);
    });
    egui::CentralPanel::default().show(context, |ui| {
        ui.add_space(14.0);
        ui.heading("Agent conversation");
        ui.label(
            RichText::new(&state.workspace)
                .small()
                .monospace()
                .color(Color32::from_gray(145)),
        );
        ui.add_space(8.0);
        chat::show(ui, &state.messages, state.busy);
    });
    prompt
}
