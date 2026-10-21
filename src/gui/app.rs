#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::Receiver;

// hide console window on Windows in release
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, StrokeKind, Visuals};

use crate::{cli::GuiArgs, gui::nixrunner, nix::tracker::NixProgressSnapshot};

pub fn gui(gui_args: GuiArgs) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(gui_args.name.as_str())
            .with_inner_size([380.0, 120.0])
            .with_min_inner_size([380.0, 120.0])
            .with_resizable(true)
            .with_maximize_button(false),
        ..Default::default()
    };

    let name = gui_args.name.clone();

    eframe::run_native(
        &name,
        options,
        Box::new(|cc| {
            let visuals = SteamDialogApp::setup_theme();
            let reciever = nixrunner::nix_runner(gui_args, cc.egui_ctx.clone());

            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(SteamDialogApp {
                header: format!("Installing {}...", name),
                detail: "Checking for available updates...".to_owned(),
                progress: 0.0,
                rx: reciever,
                error: None,
                error_expanded: false,
            }))
        }),
    )
}

struct SteamDialogApp {
    header: String,
    detail: String,
    progress: f32,
    rx: Receiver<NixProgressSnapshot>,
    error: Option<String>,
    error_expanded: bool,
}

impl SteamDialogApp {
    fn process_snapshot(&mut self, snapshot: NixProgressSnapshot) -> bool {
        self.header = snapshot.header;
        self.detail = snapshot.detail;

        if snapshot.error.is_some() {
            self.error = snapshot.error;
        }

        if self.error.is_none()
            && let Some(fraction) = snapshot.global.fraction
        {
            self.progress = fraction;
        }

        snapshot.is_finished && self.error.is_none()
    }

    fn setup_theme() -> egui::Visuals {
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
        visuals
    }
}

impl eframe::App for SteamDialogApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(snapshot) = self.rx.try_recv() {
            if self.process_snapshot(snapshot) {
                ui.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.request_repaint();
        }

        if self.error.is_some() && !self.error_expanded {
            self.error_expanded = true;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(480.0, 240.0)));
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(ui.style())
                    .inner_margin(egui::Margin::symmetric(18, 12)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 6.0);

                if let Some(err) = &self.error {
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
                                            RichText::new(err)
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
                            ui.ctx().copy_text(err.clone());
                        }
                        if ui
                            .add_sized([80.0, 24.0], egui::Button::new("Close"))
                            .clicked()
                        {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                } else {
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
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nix::tracker::GlobalProgress;

    #[test]
    fn test_process_snapshot_normal_progress() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut app = SteamDialogApp {
            header: "Installing...".to_string(),
            detail: "Starting...".to_string(),
            progress: 0.0,
            rx,
            error: None,
            error_expanded: false,
        };

        let snapshot = NixProgressSnapshot {
            global: GlobalProgress {
                fraction: Some(0.45),
                ..Default::default()
            },
            individual: None,
            header: "Downloading packages (1/2)...".to_string(),
            detail: "fetching foo".to_string(),
            error: None,
            is_finished: false,
            fraction: Some(0.45),
            builds: (0, 0, 0),
            downloads: (1, 2, 0),
        };

        let should_close = app.process_snapshot(snapshot);
        assert!(!should_close);
        assert_eq!(app.header, "Downloading packages (1/2)...");
        assert_eq!(app.detail, "fetching foo");
        assert_eq!(app.progress, 0.45);
        assert!(app.error.is_none());
    }

    #[test]
    fn test_process_snapshot_normal_finished() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut app = SteamDialogApp {
            header: "Installing...".to_string(),
            detail: "Starting...".to_string(),
            progress: 0.0,
            rx,
            error: None,
            error_expanded: false,
        };

        let snapshot = NixProgressSnapshot {
            global: GlobalProgress {
                fraction: Some(1.0),
                ..Default::default()
            },
            individual: None,
            header: "Complete".to_string(),
            detail: "done".to_string(),
            error: None,
            is_finished: true,
            fraction: Some(1.0),
            builds: (0, 0, 0),
            downloads: (0, 0, 0),
        };

        let should_close = app.process_snapshot(snapshot);
        assert!(should_close);
        assert!(app.error.is_none());
    }

    #[test]
    fn test_process_snapshot_error_mode() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut app = SteamDialogApp {
            header: "Installing...".to_string(),
            detail: "Starting...".to_string(),
            progress: 0.2,
            rx,
            error: None,
            error_expanded: false,
        };

        let snapshot = NixProgressSnapshot {
            global: GlobalProgress {
                fraction: Some(0.8),
                ..Default::default()
            },
            individual: None,
            header: "Installation failed".to_string(),
            detail: "error: build failed".to_string(),
            error: Some("error: build failed".to_string()),
            is_finished: false,
            fraction: Some(0.8),
            builds: (0, 1, 0),
            downloads: (0, 0, 0),
        };

        let should_close = app.process_snapshot(snapshot);
        assert!(!should_close);
        assert_eq!(app.header, "Installation failed");
        assert_eq!(app.error, Some("error: build failed".to_string()));
        assert_eq!(app.progress, 0.2);
    }

    #[test]
    fn test_process_snapshot_error_finished_does_not_close() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut app = SteamDialogApp {
            header: "Installing...".to_string(),
            detail: "Starting...".to_string(),
            progress: 0.0,
            rx,
            error: Some("error: already errored".to_string()),
            error_expanded: false,
        };

        let snapshot = NixProgressSnapshot {
            global: GlobalProgress {
                fraction: Some(1.0),
                ..Default::default()
            },
            individual: None,
            header: "Installation failed".to_string(),
            detail: "finished".to_string(),
            error: None,
            is_finished: true,
            fraction: Some(1.0),
            builds: (0, 0, 0),
            downloads: (0, 0, 0),
        };

        let should_close = app.process_snapshot(snapshot);
        assert!(!should_close);
        assert_eq!(app.error, Some("error: already errored".to_string()));
        assert_eq!(app.progress, 0.0);
    }
}
