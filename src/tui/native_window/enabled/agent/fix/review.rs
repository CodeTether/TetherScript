//! Git change review and validation evidence panel.

use super::{layout::Action, State};
use eframe::egui::{self, RichText};

pub(super) fn show(ui: &mut egui::Ui, state: &mut State) -> Option<Action> {
    ui.heading("3. Review evidence");
    let mut action = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.git.running, egui::Button::new("Refresh changes"))
            .clicked()
        {
            action = Some(Action::Refresh);
        }
        if state.git.running {
            ui.label("Reading Git...");
        }
    });
    output(ui, &state.git.label, &state.git.output, 300.0);
    ui.separator();
    ui.heading("4. Validate");
    ui.horizontal(|ui| {
        ui.label("Command");
        ui.text_edit_singleline(&mut state.command);
        if ui
            .add_enabled(
                !state.validation.running && !state.command.trim().is_empty(),
                egui::Button::new("Run / F6"),
            )
            .clicked()
        {
            action = Some(Action::Validate);
        }
    });
    output(ui, &state.validation.label, &state.validation.output, 220.0);
    action
}

fn output(ui: &mut egui::Ui, title: &str, text: &str, height: f32) {
    ui.label(RichText::new(title).strong());
    egui::ScrollArea::vertical()
        .max_height(height)
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(text).monospace().size(12.0))
                    .selectable(true)
                    .wrap(false),
            );
        });
}
