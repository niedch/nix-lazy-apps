use cli::{Cli, CliParser, Commands};

mod cli;
mod gui;
mod nix;

pub fn create_nix_run_command(gui_args: &crate::cli::GuiArgs) -> std::process::Command {
    let mut cmd = std::process::Command::new("nix");
    cmd.arg("run");

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Gui(gui_args) => {
            println!(
                "Starting GUI for '{}' with command: '{}'",
                gui_args.name, gui_args.flake_output
            );

            match gui::loading_view::gui(gui_args.clone())? {
                gui::GuiResult::Success => {
                    let mut cmd = create_nix_run_command(&gui_args);
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::CommandExt;
                        let err = cmd.exec();
                        eprintln!("Failed to execute process: {err}");
                        std::process::exit(1);
                    }
                    #[cfg(not(unix))]
                    {
                        let mut child = cmd.spawn()?;
                        let status = child.wait()?;
                        let code = status.code().unwrap_or(1);
                        std::process::exit(code);
                    }
                }
                gui::GuiResult::Cancelled => {
                    std::process::exit(0);
                }
                gui::GuiResult::Error(err) => {
                    gui::error_view::gui(&gui_args, &err)?;
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::GuiArgs;

    fn get_command_args(cmd: &std::process::Command) -> Vec<String> {
        cmd.get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    fn get_command_env(cmd: &std::process::Command, key: &str) -> Option<String> {
        cmd.get_envs()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.map(|v| v.to_string_lossy().into_owned()))
    }

    #[test]
    fn test_create_nix_run_command_default() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: false,
            args: vec![],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(cmd.get_program(), "nix");
        assert_eq!(get_command_args(&cmd), vec!["run", "nixpkgs#hello"]);
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_run_command_unfree() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#discord".to_string(),
            name: "discord".to_string(),
            unfree: true,
            impure: false,
            args: vec![],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(
            get_command_args(&cmd),
            vec!["run", "--impure", "nixpkgs#discord"]
        );
        assert_eq!(
            get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_create_nix_run_command_impure() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: true,
            args: vec![],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(
            get_command_args(&cmd),
            vec!["run", "--impure", "nixpkgs#hello"]
        );
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_run_command_unfree_and_impure() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#steam".to_string(),
            name: "Steam".to_string(),
            unfree: true,
            impure: true,
            args: vec![],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(
            get_command_args(&cmd),
            vec!["run", "--impure", "nixpkgs#steam"]
        );
        assert_eq!(
            get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_create_nix_run_command_trailing_args() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#hello".to_string(),
            name: "hello".to_string(),
            unfree: false,
            impure: false,
            args: vec!["--flag".to_string(), "val".to_string()],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(
            get_command_args(&cmd),
            vec!["run", "nixpkgs#hello", "--", "--flag", "val"]
        );
        assert_eq!(get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"), None);
    }

    #[test]
    fn test_create_nix_run_command_all_combined() {
        let gui_args = GuiArgs {
            flake_output: "nixpkgs#steam".to_string(),
            name: "Steam".to_string(),
            unfree: true,
            impure: true,
            args: vec!["-silent".to_string(), "extra".to_string()],
        };

        let cmd = create_nix_run_command(&gui_args);
        assert_eq!(
            get_command_args(&cmd),
            vec!["run", "--impure", "nixpkgs#steam", "--", "-silent", "extra"]
        );
        assert_eq!(
            get_command_env(&cmd, "NIXPKGS_ALLOW_UNFREE"),
            Some("1".to_string())
        );
    }
}
