use crate::cli::nix_lazy_apps::{Cli, Commands};
pub use clap::Parser as CliParser;

#[test]
fn test_cli_gui_default_flags() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#hello",
        "--name",
        "hello",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert_eq!(gui.flake_output, "nixpkgs#hello");
        assert_eq!(gui.name, "hello");
        assert!(!gui.unfree);
        assert!(!gui.impure);
        assert!(gui.args.is_empty());
    }
}

#[test]
fn test_cli_gui_unfree() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#discord",
        "--name",
        "discord",
        "--unfree",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert!(gui.unfree);
        assert!(!gui.impure);
        assert!(gui.args.is_empty());
    }
}

#[test]
fn test_cli_gui_impure() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#hello",
        "--name",
        "hello",
        "--impure",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert!(!gui.unfree);
        assert!(gui.impure);
        assert!(gui.args.is_empty());
    }
}

#[test]
fn test_cli_gui_trailing_args_with_double_dash() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#hello",
        "--name",
        "hello",
        "--",
        "--verbose",
        "-t",
        "pos_arg",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert_eq!(gui.args, vec!["--verbose", "-t", "pos_arg"]);
    }
}

#[test]
fn test_cli_gui_empty_double_dash() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#hello",
        "--name",
        "hello",
        "--",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert_eq!(gui.flake_output, "nixpkgs#hello");
        assert_eq!(gui.name, "hello");
        assert!(gui.args.is_empty());
    }
}

#[test]
fn test_cli_gui_trailing_positional_args() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--flake-output",
        "nixpkgs#hello",
        "--name",
        "hello",
        "arg1",
        "arg2",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert_eq!(gui.args, vec!["arg1", "arg2"]);
    }
}

#[test]
fn test_cli_gui_all_combined() {
    let cli = Cli::try_parse_from([
        "nix-lazy-apps",
        "gui",
        "--unfree",
        "--impure",
        "--flake-output",
        "nixpkgs#steam",
        "--name",
        "Steam",
        "--",
        "-silent",
        "extra",
    ]);
    assert!(cli.is_ok());
    if let Ok(Cli {
        command: Commands::Gui(gui),
    }) = cli
    {
        assert_eq!(gui.flake_output, "nixpkgs#steam");
        assert_eq!(gui.name, "Steam");
        assert!(gui.unfree);
        assert!(gui.impure);
        assert_eq!(gui.args, vec!["-silent", "extra"]);
    }
}
