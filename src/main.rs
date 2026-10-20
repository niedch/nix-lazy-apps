use std::process::Stdio;

use cli::{Cli, CliParser, Commands};
use tokio::{io::AsyncBufReadExt, process::Command};

use crate::nix::tracker::NixStateTracker;

mod cli;
mod gui;
mod nix;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cli::parse();

    match args.command {
        Commands::Gui(gui_args) => {
            println!(
                "Starting GUI for '{}' with command: '{}'",
                gui_args.name, gui_args.flake_output
            );

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
                    .spawn() {
                        Ok(child) => child,
                        Err(err) => {
                            eprintln!("Error spawning child prcess: {err}");
                            return;
                        },
                    };


                if let Some(stderr) = child.stderr.take() {
                    let mut reader = tokio::io::BufReader::new(stderr).lines();

                    while let Ok(Some(line)) = reader.next_line().await {
                        let snapshot = tracker.process_line(&line);
                        tx.send(snapshot).expect("To work"); // Send snapshot to egui over mpsc / watch channel
                    }
                }
            });

            gui::app::gui(gui_args.name, rx)?;
            Ok(())
        }
    }
}
