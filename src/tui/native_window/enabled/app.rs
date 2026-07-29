//! Native dashboard application lifecycle.

use eframe::egui;

use super::{footer, header, model::Dashboard, sidebar};

pub(super) struct DashboardApp {
    pub dashboard: Dashboard,
}

impl DashboardApp {
    pub fn new(dashboard: Dashboard) -> Self {
        Self { dashboard }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if context.input(|input| input.key_pressed(egui::Key::Escape)) {
            context.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        header::show(context, &self.dashboard);
        sidebar::show(context, &self.dashboard);
        footer::show(context, &self.dashboard);
        egui::CentralPanel::default().show(context, |ui| {
            ui.add_space(24.0);
            ui.heading("Release checklist");
            ui.label("Select a task to update its completion state.");
            ui.add_space(18.0);
            super::tasks::list(ui, &mut self.dashboard.tasks);
        });
    }
}
