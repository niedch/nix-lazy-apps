#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::{Receiver, Sender};

// hide console window on Windows in release
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, StrokeKind};

use crate::{
    cli::GuiArgs,
    gui::{nixrunner, theme::setup_theme},
    nix::tracker::NixProgressSnapshot,
};

pub fn gui(gui_args: GuiArgs) -> Result<Option<String>, eframe::Error> {
    let (error_tx, error_rx) = std::sync::mpsc::channel();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("nix-lazy-apps")
            .with_title(gui_args.name.as_str())
            .with_inner_size([380.0, 120.0])
            .with_min_inner_size([380.0, 120.0])
            .with_max_inner_size([380.0, 120.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    let name = gui_args.name.clone();

    eframe::run_native(
        "nix-lazy-apps",
        options,
        Box::new(move |cc| {
            let visuals = setup_theme();
            let reciever = nixrunner::nix_runner(gui_args, cc.egui_ctx.clone());

            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(LoadingDialogApp {
                header: format!("Installing {}...", name),
                detail: "Checking for available updates...".to_owned(),
                progress: 0.0,
                rx: reciever,
                error: None,
                error_tx,
            }))
        }),
    )?;

    Ok(error_rx.try_recv().ok())
}

pub struct LoadingDialogApp {
    pub header: String,
    pub detail: String,
    pub progress: f32,
    pub rx: Receiver<NixProgressSnapshot>,
    pub error: Option<String>,
    pub error_tx: Sender<String>,
}

impl LoadingDialogApp {
    pub fn process_snapshot(&mut self, snapshot: NixProgressSnapshot) -> bool {
        self.header = snapshot.header;
        self.detail = snapshot.detail;

        if snapshot.error.is_some() {
            self.error = snapshot.error;
            return true;
        }

        if self.error.is_none()
            && let Some(fraction) = snapshot.global.fraction
        {
            self.progress = fraction;
        }

        snapshot.is_finished
    }

    fn receive_snapshot(&mut self, ui: &mut egui::Ui) {
        while let Ok(snapshot) = self.rx.try_recv() {
            if !self.process_snapshot(snapshot) {
                ui.request_repaint();
                return;
            }

            if let Some(err) = &self.error {
                let _ = self.error_tx.send(err.clone());
            }
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for LoadingDialogApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_snapshot(ui);

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

                ui.add(
                    egui::Label::new(
                        RichText::new(&self.detail)
                            .size(11.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                    )
                    .truncate(),
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
