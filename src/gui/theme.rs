use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

pub fn setup_theme() -> egui::Visuals {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(38, 38, 38);
    visuals.window_fill = Color32::from_rgb(38, 38, 38);
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 20);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(209, 209, 209));

    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(56, 56, 56);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(56, 56, 56);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(70, 70, 70));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(198, 198, 198));
    visuals.widgets.inactive.corner_radius = CornerRadius::same(2);
    visuals.widgets.inactive.expansion = 0.0;

    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(75, 75, 75);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(75, 75, 75);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(110, 110, 110));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(2);
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(45, 45, 45);
    visuals.widgets.active.bg_fill = Color32::from_rgb(45, 45, 45);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140));
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(2);
    visuals.widgets.active.expansion = 0.0;
    visuals
}
