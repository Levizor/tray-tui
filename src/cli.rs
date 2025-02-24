use clap::value_parser;
use clap::Parser;
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "CONFIG_PATH", value_parser = value_parser!(std::path::PathBuf))]
    pub config_path: Option<std::path::PathBuf>,

    /// Prints debug information to app.log file
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub debug: bool,

    /// Generates completion scripts for the specified shell
    #[arg(long, value_name = "SHELL", value_enum)]
    pub completions: Option<Shell>,
}
