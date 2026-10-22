use tokio::process::Command;

use crate::{
    cli::GuiArgs, gui::nix_command::nixrunner::create_nix_command,
};

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
    assert_eq!(cmd.as_std().get_program(), "nix");
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
