//! Fix Runner identity and live provider status header.

use super::super::super::state::AgentState;
use eframe::egui::{self, Color32, RichText};

pub(super) fn show(context: &egui::Context, agent: &AgentState) {
    egui::TopBottomPanel::top("fix_header")
        .exact_height(72.0)
        .show(context, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("FIX")
                        .size(24.0)
                        .color(Color32::from_rgb(75, 210, 145)),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new("Tether Fix Runner").size(23.0).strong());
                    ui.label(RichText::new(&agent.status).color(Color32::from_gray(155)));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(&agent.model).monospace());
                    ui.label("MODEL");
                });
            });
        });
}
