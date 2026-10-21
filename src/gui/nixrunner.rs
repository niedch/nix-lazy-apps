use std::{process::Stdio, sync::mpsc::Receiver};

use tokio::{io::AsyncBufReadExt, process::Command};

use crate::{
    cli::GuiArgs,
    nix::tracker::{NixProgressSnapshot, NixStateTracker},
};

pub fn create_nix_command(gui_args: &GuiArgs) -> Command {
    let mut cmd = Command::new("nix");
    cmd.args(["run", "--log-format", "internal-json"]);

    if gui_args.impure || gui_args.unfree {
        cmd.arg("--impure");
    }

    if gui_args.unfree {
        cmd.env("NIXPKGS_ALLOW_UNFREE", "1");
    }

    cmd.arg(&gui_args.flake_output);

    if !gui_args.args.is_empty() {
        cmd.arg("--");
        cmd.args(&gui_args.args);
    }

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
        let mut child = match spawn_nix_run(&gui_args) {
            Ok(child) => child,
            Err(err) => {
                eprintln!("Error spawning child process: {err}");
                return;
            }
        };

        if let Some(stderr) = child.stderr.take() {
            let mut reader = tokio::io::BufReader::new(stderr).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                let snapshot = tracker.process_line(&line);
                if tx.send(snapshot).is_err() {
                    break;
                }
                egui_ctx.request_repaint();
            }
        }
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_command_args(cmd: &Command) -> Vec<String> {
        cmd.as_std()
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    fn get_command_env(cmd: &Command, key: &str) -> Option<String> {
        cmd.as_std()
            .get_envs()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.map(|v| v.to_string_lossy().into_owned()))
    }

    #[test]
    fn test_create_nix_command_default() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: false,
            args: vec![],
        };

        let cmd = create_nix_command(&gui_args);
        let args = get_command_args(&cmd);
        assert_eq!(
            args,
            vec!["run", "--log-format", "internal-json", "nixpkgs#hello"]
        );
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_command_unfree() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#discord".to_string(),
            name: "discord".to_string(),
            unfree: true,
            impure: false,
            args: vec![],
        };

        let cmd = create_nix_command(&gui_args);
        let args = get_command_args(&cmd);
        assert_eq!(
            args,
            vec![
                "run",
                "--log-format",
                "internal-json",
                "--impure",
                "nixpkgs#discord"
            ]
        );
        assert_eq!(
            get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_create_nix_command_impure() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: true,
            args: vec![],
        };

        let cmd = create_nix_command(&gui_args);
        let args = get_command_args(&cmd);
        assert_eq!(
            args,
            vec![
                "run",
                "--log-format",
                "internal-json",
                "--impure",
                "nixpkgs#hello"
            ]
        );
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_command_trailing_args() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: false,
            args: vec!["--flag".to_string(), "val".to_string()],
        };

        let cmd = create_nix_command(&gui_args);
        let args = get_command_args(&cmd);
        assert_eq!(
            args,
            vec![
                "run",
                "--log-format",
                "internal-json",
                "nixpkgs#hello",
                "--",
                "--flag",
                "val"
            ]
        );
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_command_unfree_impure_and_trailing_args() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#steam".to_string(),
            name: "Steam".to_string(),
            unfree: true,
            impure: true,
            args: vec!["-silent".to_string()],
        };

        let cmd = create_nix_command(&gui_args);
        let args = get_command_args(&cmd);
        assert_eq!(
            args,
            vec![
                "run",
                "--log-format",
                "internal-json",
                "--impure",
                "nixpkgs#steam",
                "--",
                "-silent"
            ]
        );
        assert_eq!(
            get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"),
            Some("1".to_string())
        );
    }
}
