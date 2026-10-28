use clap::{Args};

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
