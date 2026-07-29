//! Native conversation transcript rendering.

use eframe::egui::{self, Color32, RichText};

use super::model::Message;

pub(super) fn show(ui: &mut egui::Ui, messages: &[Message], busy: bool) {
    let recent = messages.len().saturating_sub(60);
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if recent > 0 {
                ui.label(format!(
                    "Showing the latest 60 of {} messages",
                    messages.len()
                ));
            }
            for message in &messages[recent..] {
                bubble(ui, message);
                ui.add_space(10.0);
            }
            if busy {
                ui.label("Agent is working with the provider and tools…");
            }
        });
}

fn bubble(ui: &mut egui::Ui, message: &Message) {
    let user = message.role == "user";
    let color = if user {
        Color32::from_rgb(31, 72, 117)
    } else {
        Color32::from_rgb(24, 37, 55)
    };
    let label = if user { "YOU" } else { message.role.as_str() };
    egui::Frame::none()
        .fill(color)
        .rounding(10.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_max_width(660.0);
            ui.label(
                RichText::new(label.to_uppercase())
                    .small()
                    .strong()
                    .color(Color32::from_rgb(120, 185, 255)),
            );
            ui.label(RichText::new(&message.text).size(15.0));
        });
}
