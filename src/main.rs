use cli::{Cli, CliParser, Commands};

mod cli;
mod gui;
mod nix;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Gui(gui_args) => {
            println!(
                "Starting GUI for '{}' with command: '{}'",
                gui_args.name, gui_args.flake_output
            );

            if let Some(err) = gui::loading_view::gui(gui_args.clone())? {
                gui::error_view::gui(&gui_args, &err)?;
            }
            Ok(())
        }
    }
}
