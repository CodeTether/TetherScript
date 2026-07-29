//! Native agent identity, provider state, and model header.

use eframe::egui::{self, Color32, RichText};

use super::super::state::AgentState;

pub(super) fn show(context: &egui::Context, state: &AgentState) {
    egui::TopBottomPanel::top("agent_header")
        .exact_height(72.0)
        .show(context, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("◈")
                        .size(30.0)
                        .color(Color32::from_rgb(88, 166, 255)),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new("tetherscript agent").size(23.0).strong());
                    ui.label(RichText::new(&state.status).color(Color32::from_gray(150)));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(&state.model).monospace());
                    ui.label("MODEL");
                });
            });
        });
}
