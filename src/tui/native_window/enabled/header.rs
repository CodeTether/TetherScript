//! Branded native dashboard header.

use eframe::egui::{self, Color32, RichText};

use super::{model::Dashboard, panels};

pub(super) fn show(context: &egui::Context, dashboard: &Dashboard) {
    egui::TopBottomPanel::top("header")
        .exact_height(76.0)
        .show(context, |ui| {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("◈")
                        .size(30.0)
                        .color(Color32::from_rgb(88, 166, 255)),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(&dashboard.title).size(24.0).strong());
                    ui.label(RichText::new("Native workspace").color(Color32::from_gray(150)));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(panels::progress_label(dashboard));
                });
            });
        });
}
