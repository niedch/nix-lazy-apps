#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, StrokeKind, Visuals};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Steam")
            .with_inner_size([380.0, 120.0])
            .with_min_inner_size([380.0, 120.0])
            .with_max_inner_size([380.0, 120.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    eframe::run_native(
        "Lazy App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut visuals = Visuals::dark();
            visuals.panel_fill = Color32::from_rgb(38, 38, 38);
            visuals.window_fill = Color32::from_rgb(38, 38, 38);
            visuals.extreme_bg_color = Color32::from_rgb(20, 20, 20);
            visuals.widgets.noninteractive.fg_stroke =
                Stroke::new(1.0, Color32::from_rgb(209, 209, 209));

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

            cc.egui_ctx.set_theme(egui::Theme::Dark);
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::<SteamDialogApp>::default())
        }),
    )
}

struct SteamDialogApp {
    header: String,
    detail: String,
    progress: f32,
    speed: f32,
}

impl Default for SteamDialogApp {
    fn default() -> Self {
        Self {
            header: "Updating Steam...".to_owned(),
            detail: "Checking for available updates...".to_owned(),
            progress: 0.0,
            speed: 0.15,
        }
    }
}

impl eframe::App for SteamDialogApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.speed > 0.0 && self.progress < 1.0 {
            let dt = ui.input(|i| i.stable_dt);
            self.progress = (self.progress + self.speed * dt).min(1.0);
            ui.ctx().request_repaint();
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(ui.style())
                    .inner_margin(egui::Margin::symmetric(18, 12)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 6.0);

                ui.label(
                    RichText::new(&self.header)
                        .size(13.0)
                        .color(Color32::from_rgb(220, 220, 220)),
                );

                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 14.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect(
                    rect,
                    CornerRadius::ZERO,
                    Color32::from_rgb(20, 20, 20),
                    Stroke::new(1.0, Color32::from_rgb(70, 70, 70)),
                    StrokeKind::Inside,
                );
                let inner_rect = rect.shrink(1.0);
                if self.progress > 0.0 {
                    let fill_width = inner_rect.width() * self.progress.clamp(0.0, 1.0);
                    let fill_rect = egui::Rect::from_min_size(
                        inner_rect.min,
                        egui::vec2(fill_width, inner_rect.height()),
                    );
                    ui.painter().rect_filled(
                        fill_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgb(56, 107, 153),
                    );
                }

                ui.label(
                    RichText::new(&self.detail)
                        .size(11.0)
                        .color(Color32::from_rgb(150, 150, 150)),
                );

                ui.add_space(8.0);

                ui.vertical_centered(|ui| {
                    if ui
                        .add_sized([80.0, 24.0], egui::Button::new("Cancel"))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
    }
}
