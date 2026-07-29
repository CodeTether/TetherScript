//! Interactive native task cards.

use eframe::egui::{self, Color32, RichText, Stroke};

use super::model::Task;

pub(super) fn list(ui: &mut egui::Ui, tasks: &mut [Task]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for task in tasks {
            card(ui, task);
            ui.add_space(10.0);
        }
    });
}

fn card(ui: &mut egui::Ui, task: &mut Task) {
    let fill = if task.done {
        Color32::from_rgb(22, 55, 48)
    } else {
        Color32::from_rgb(25, 36, 54)
    };
    let frame = egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(10.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(47, 67, 94)))
        .inner_margin(egui::Margin::symmetric(16.0, 14.0));
    let response = frame
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.checkbox(&mut task.done, "");
                ui.vertical(|ui| {
                    let title = RichText::new(&task.title).size(17.0).strong();
                    ui.label(if task.done {
                        title.strikethrough()
                    } else {
                        title
                    });
                    if !task.detail.is_empty() {
                        ui.label(RichText::new(&task.detail).color(Color32::from_gray(155)));
                    }
                });
            });
        })
        .response
        .interact(egui::Sense::click());
    if response.clicked() {
        task.done = !task.done;
    }
}
