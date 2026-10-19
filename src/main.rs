use cli::{Cli, CliParser, Commands};

mod cli;
mod gui;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Gui(gui_args) => {
            println!(
                "Starting GUI for '{}' with command: '{}'",
                gui_args.name, gui_args.flake_output
            );

            gui::app::gui(gui_args.name).expect("Test")
        }
    }
}
