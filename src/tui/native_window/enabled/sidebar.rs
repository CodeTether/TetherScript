//! Native workspace sidebar.

use eframe::egui::{self, Color32, RichText};

use super::model::Dashboard;

pub(super) fn show(context: &egui::Context, dashboard: &Dashboard) {
    egui::SidePanel::left("sidebar")
        .exact_width(240.0)
        .show(context, |ui| {
            ui.add_space(28.0);
            ui.label(
                RichText::new("WORKSPACE")
                    .small()
                    .color(Color32::from_rgb(88, 166, 255)),
            );
            ui.heading("Build queue");
            ui.label("A real native interface driven by tetherscript data.");
            ui.add_space(24.0);
            let complete = dashboard.tasks.iter().filter(|task| task.done).count();
            let progress = complete as f32 / dashboard.tasks.len().max(1) as f32;
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            ui.add_space(28.0);
            ui.label(RichText::new("SHORTCUTS").small().strong());
            ui.label("Click a task   Toggle completion");
            ui.label("Esc             Close window");
        });
}
