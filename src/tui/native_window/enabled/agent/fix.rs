//! ROI-focused repository task runner built on the shared agent RPC client.

#[path = "fix/commands.rs"]
mod commands;
#[path = "fix/header.rs"]
mod header;
#[path = "fix/job.rs"]
mod job;
#[path = "fix/layout.rs"]
mod layout;
#[path = "fix/prompt.rs"]
mod prompt;
#[path = "fix/review.rs"]
mod review;
#[path = "fix/state.rs"]
mod state;
#[path = "fix/task.rs"]
mod task;
#[cfg(test)]
#[path = "fix/tests.rs"]
mod tests;
use eframe::egui;

use super::AgentApp;

pub(super) use state::State;

pub(super) fn update(app: &mut AgentApp, context: &egui::Context) {
    if app.fix.poll(context, &app.state) {
        commands::refresh(&mut app.fix.git, context);
    }
    match layout::show(context, &mut app.fix, &app.state) {
        Some(layout::Action::RunTask) => {
            let request = prompt::build(&app.fix.task);
            super::super::actions::submit(app, request);
            app.fix.started(app.state.messages.len());
        }
        Some(layout::Action::Refresh) => commands::refresh(&mut app.fix.git, context),
        Some(layout::Action::Validate) => {
            commands::validate(&mut app.fix.validation, &app.fix.command, context)
        }
        None => {}
    }
}
