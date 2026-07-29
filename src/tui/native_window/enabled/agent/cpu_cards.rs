//! Executable and selected-instruction state cards.

use eframe::egui::{self, Color32, RichText};

use super::super::disassembly::Snapshot;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &Snapshot) {
    ui.horizontal_wrapped(|ui| {
        card(ui, "ARCH", &snapshot.architecture);
        card(ui, "ENTRY", &format!("0x{:016X}", snapshot.entry));
        card(ui, "TEXT BASE", &format!("0x{:016X}", snapshot.base));
        card(ui, "TEXT SIZE", &format!("0x{:X}", snapshot.size));
        card(
            ui,
            "SELECTED",
            &format!("0x{:016X}", snapshot.selected_address()),
        );
    });
}

fn card(ui: &mut egui::Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(Color32::from_rgb(20, 32, 49))
        .rounding(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .small()
                    .color(Color32::from_rgb(88, 166, 255)),
            );
            ui.label(RichText::new(value).monospace().size(12.0));
        });
}
