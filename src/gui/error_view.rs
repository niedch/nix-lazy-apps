use crate::{cli::gui_args::GuiArgs, gui::theme::setup_theme};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};

/// Displays a dedicated error window for an application failure.
pub fn gui(gui_args: &GuiArgs, error: &str) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("nix-lazy-apps")
            .with_title(gui_args.name.as_str())
            .with_inner_size([480.0, 240.0])
            .with_min_inner_size([480.0, 240.0])
            .with_max_inner_size([480.0, 240.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    let error_text = error.to_owned();
    let name = gui_args.name.clone();

    eframe::run_native(
        "nix-lazy-apps",
        options,
        Box::new(move |cc| {
            let visuals = setup_theme();
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(ErrorDialogApp::new(name, error_text)))
        }),
    )
}

pub struct ErrorDialogApp {
    pub header: String,
    pub error: String,
}

impl ErrorDialogApp {
    pub fn new(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            header: format!("Failed to run {}", name.into()),
            error: error.into(),
        }
    }
}

impl eframe::App for ErrorDialogApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
                        .color(Color32::from_rgb(235, 75, 75)),
                );

                let available_height = ui.available_height();
                let button_height = 24.0;
                let spacing = ui.spacing().item_spacing.y;
                let box_height = (available_height - button_height - spacing - 14.0).max(40.0);

                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 20, 20))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(70, 70, 70)))
                    .corner_radius(CornerRadius::same(2))
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(box_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&self.error)
                                            .monospace()
                                            .size(11.0)
                                            .color(Color32::from_rgb(209, 209, 209)),
                                    )
                                    .wrap(),
                                );
                            });
                    });

                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                    let total_width = 100.0 + 80.0 + 8.0;
                    let space = (ui.available_width() - total_width) / 2.0;
                    if space > 0.0 {
                        ui.add_space(space);
                    }
                    if ui
                        .add_sized([100.0, 24.0], egui::Button::new("Copy Error"))
                        .clicked()
                    {
                        ui.ctx().copy_text(self.error.clone());
                    }
                    if ui
                        .add_sized([80.0, 24.0], egui::Button::new("Close"))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
    }
}
