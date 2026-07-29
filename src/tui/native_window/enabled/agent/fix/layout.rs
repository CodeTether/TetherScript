//! Two-pane Fix Runner workflow layout.

use eframe::egui;

use super::super::super::state::AgentState;
use super::{header, review, task, State};

pub(crate) enum Action {
    RunTask,
    Refresh,
    Validate,
}

pub(super) fn show(context: &egui::Context, fix: &mut State, agent: &AgentState) -> Option<Action> {
    header::show(context, agent);
    let mut action = None;
    egui::SidePanel::right("fix_evidence")
        .default_width(650.0)
        .resizable(true)
        .show(context, |ui| {
            ui.add_space(14.0);
            action = review::show(ui, fix);
        });
    egui::CentralPanel::default().show(context, |ui| {
        ui.add_space(14.0);
        if task::show(ui, fix, agent) {
            action = Some(Action::RunTask);
        }
    });
    let f6 = context.input(|input| input.key_pressed(egui::Key::F6));
    if action.is_none() && f6 && !fix.validation.running && !fix.command.trim().is_empty() {
        action = Some(Action::Validate);
    }
    let f5 = context.input(|input| input.key_pressed(egui::Key::F5));
    if action.is_none() && f5 && !agent.busy && !fix.task.trim().is_empty() {
        action = Some(Action::RunTask);
    }
    action
}
