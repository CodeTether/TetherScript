//! Scrollable machine-instruction rows.

use eframe::egui::{self, RichText};

use super::super::disassembly::Snapshot;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &mut Snapshot) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, 20.0, snapshot.rows.len(), |ui, visible| {
            egui::Grid::new("disassembly_rows")
                .num_columns(3)
                .striped(true)
                .spacing([18.0, 3.0])
                .show(ui, |ui| {
                    for index in visible {
                        let row = &snapshot.rows[index];
                        let location = format!("{:016X}  +{:06X}", row.address, row.offset);
                        let selected = snapshot.selected == index;
                        if ui.selectable_label(selected, mono(&location)).clicked() {
                            snapshot.selected = index;
                        }
                        ui.label(mono(&row.bytes));
                        ui.label(mono(&row.assembly));
                        ui.end_row();
                    }
                });
        });
}

fn mono(text: &str) -> RichText {
    RichText::new(text).monospace().size(12.0)
}
