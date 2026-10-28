use std::{process::Stdio, sync::mpsc::Receiver};

use tokio::{io::AsyncBufReadExt, process::Command};

use crate::{
    cli::gui_args::GuiArgs, gui::nix_command::child_guard::ChildGuard, nix::tracker::{NixProgressSnapshot, NixStateTracker},
};

pub fn create_nix_command(gui_args: &GuiArgs) -> Command {
    let mut cmd = Command::new("nix");
    cmd.args(["build", "--no-link", "--log-format", "internal-json"]);

    if gui_args.impure || gui_args.unfree {
        cmd.arg("--impure");
    }

    if gui_args.unfree {
        cmd.env("NIXPKGS_ALLOW_UNFREE", "1");
    }

    cmd.arg(&gui_args.flake_output);

    cmd
}

pub fn spawn_nix_run(gui_args: &GuiArgs) -> std::io::Result<tokio::process::Child> {
    create_nix_command(gui_args).stderr(Stdio::piped()).spawn()
}

pub fn nix_runner(
    gui_args: GuiArgs,
    egui_ctx: eframe::egui::Context,
) -> Receiver<NixProgressSnapshot> {
    let (tx, rx) = std::sync::mpsc::channel();

    tokio::spawn(async move {
        let mut tracker = NixStateTracker::new();
        let child = match spawn_nix_run(&gui_args) {
            Ok(child) => child,
            Err(err) => {
                eprintln!("Error spawning child process: {err}");
                return;
            }
        };

        let mut guard = ChildGuard {
            child,
            disarmed: false,
        };

        if let Some(stderr) = guard.child.stderr.take() {
            let mut reader = tokio::io::BufReader::new(stderr).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                let snapshot = tracker.process_line(&line);
                println!("Received snapshot {:?}", snapshot);
                if snapshot.is_finished {
                    guard.disarmed = true;
                }
                if tx.send(snapshot).is_err() {
                    break;
                }
                egui_ctx.request_repaint();
            }
        }
    });

    rx
}
