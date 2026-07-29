//! Native dashboard colors, spacing, and interaction styling.

use eframe::egui::{self, Color32};

pub(super) fn install(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(11, 18, 30);
    visuals.window_fill = Color32::from_rgb(11, 18, 30);
    visuals.extreme_bg_color = Color32::from_rgb(16, 27, 43);
    visuals.selection.bg_fill = Color32::from_rgb(47, 112, 180);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(24, 38, 58);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 57, 84);
    visuals.widgets.active.bg_fill = Color32::from_rgb(47, 112, 180);
    context.set_visuals(visuals);
    let mut style = (*context.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    context.set_style(style);
}
