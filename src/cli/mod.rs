use clap::{Args, Parser, Subcommand};

pub use clap::Parser as CliParser;

#[derive(Parser, Debug)]
#[command(
    name = "nix-lazy-apps",
    version,
    about = "A simple GUI to show a loading indicator till the nix run works"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Starts the gui
    #[command(about = "Starts the gui")]
    Gui(GuiArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GuiArgs {
    /// The command to execute while showing the GUI
    #[arg(long, help = "The flake output to execute while showing GUI")]
    pub flake_output: String,

    /// Name of the application
    #[arg(long, help = "Name of the application")]
    pub name: String,
}
