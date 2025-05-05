use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the file that contains the combat info (, use - to read from stdin (must not be a tty))
    #[arg(short, long)]
    pub combat_file: PathBuf,

    /// The output path to save an encounter to
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print the combat state to stdout
    #[arg(long)]
    pub stdout: bool,

    /// The path to the file that holds the information about the player characters
    #[arg(long, short)]
    pub player_characters: Option<PathBuf>,
}
