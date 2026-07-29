//! Multiline native agent prompt composer.

use eframe::egui;

pub(super) fn show(ui: &mut egui::Ui, input: &mut String, busy: bool) -> Option<String> {
    ui.separator();
    let editor = egui::TextEdit::multiline(input)
        .hint_text("Ask the agent to inspect, edit, test, or explain the workspace…")
        .desired_rows(3)
        .lock_focus(true);
    let response = ui.add_sized([ui.available_width(), 72.0], editor);
    let shortcut = response.has_focus()
        && ui.input(|state| state.key_pressed(egui::Key::Enter) && !state.modifiers.shift);
    let clicked = ui
        .add_enabled(
            !busy && !input.trim().is_empty(),
            egui::Button::new("Send  Enter"),
        )
        .clicked();
    if !busy && (clicked || shortcut) {
        let prompt = input.trim().to_string();
        input.clear();
        Some(prompt)
    } else {
        None
    }
}
