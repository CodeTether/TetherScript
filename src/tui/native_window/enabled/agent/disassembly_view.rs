//! CPU state cards and native disassembly table.

#[path = "cpu_cards.rs"]
mod cards;
#[path = "disassembly_rows.rs"]
mod rows;

use eframe::egui::{self, Color32, RichText};

use super::disassembly::Snapshot;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &mut Snapshot) {
    ui.heading("CPU / executable state");
    cards::show(ui, snapshot);
    ui.add_space(12.0);
    egui::Grid::new("disassembly_header")
        .num_columns(3)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            heading(ui, "Address / Offset");
            heading(ui, "Machine Bytes (Hex)");
            heading(ui, "Assembly Instruction");
            ui.end_row();
        });
    ui.separator();
    rows::show(ui, snapshot);
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).strong().color(Color32::from_gray(190)));
}
