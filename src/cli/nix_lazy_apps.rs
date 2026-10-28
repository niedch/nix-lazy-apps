use clap::{Parser, Subcommand};
use crate::cli::gui_args::GuiArgs;

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
