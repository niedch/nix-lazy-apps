use clap::{Args, Parser, Subcommand};

pub use clap::Parser as CliParser;

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(
    name = "nix-lazy-apps",
    version,
    about = "A simple GUI to show a loading indicator till the nix run works"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Starts the gui
    #[command(about = "Starts the gui")]
    Gui(GuiArgs),
}

#[derive(Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct GuiArgs {
    /// The command to execute while showing the GUI
    #[arg(long, help = "The flake output to execute while showing GUI")]
    pub flake_output: String,

    /// Name of the application
    #[arg(long, help = "Name of the application")]
    pub name: String,

    #[arg(long, help = "Allow unfree packages")]
    pub unfree: bool,

    #[arg(long, help = "Allow impure evaluation")]
    pub impure: bool,

    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "Arguments to pass to the application"
    )]
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
