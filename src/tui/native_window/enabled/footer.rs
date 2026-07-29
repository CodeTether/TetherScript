//! Native dashboard status footer.

use eframe::egui;

use super::model::Dashboard;

pub(super) fn show(context: &egui::Context, dashboard: &Dashboard) {
    egui::TopBottomPanel::bottom("footer")
        .exact_height(36.0)
        .show(context, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label("●");
                ui.label(&dashboard.status);
            });
        });
}
