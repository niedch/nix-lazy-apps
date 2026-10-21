use std::{process::Stdio, sync::mpsc::Receiver};

use tokio::{io::AsyncBufReadExt, process::Command};

use crate::{cli::GuiArgs, nix::tracker::{NixProgressSnapshot, NixStateTracker}};

pub fn nix_runner(gui_args: GuiArgs, egui_ctx: eframe::egui::Context) -> Receiver<NixProgressSnapshot> {
    let (tx, rx) = std::sync::mpsc::channel();

    tokio::spawn(async move {
        let mut tracker = NixStateTracker::new();
        let mut child = match Command::new("nix")
            .args([
                "run",
                "--log-format",
                "internal-json",
                gui_args.flake_output.as_str(),
            ])
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                eprintln!("Error spawning child prcess: {err}");
                return;
            }
        };

        if let Some(stderr) = child.stderr.take() {
            let mut reader = tokio::io::BufReader::new(stderr).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                let snapshot = tracker.process_line(&line);
                tx.send(snapshot).expect("To work"); // Send snapshot to egui over mpsc / watch channel
                egui_ctx.request_repaint();
            }
        }
    });

    rx
}
