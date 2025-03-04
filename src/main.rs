#![allow(dead_code)]
use std::fs;

use clap::Parser;
use color_eyre::Result;
mod combat;
mod tui;

use crate::tui::{app, cli, terminal};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    // create tui
    let mut terminal = terminal::init()?;
    let mut app = app::App::new(&args)?;
    let tracker = app.run(&mut terminal)?;
    if let Err(err) = terminal::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    if args.stdout {
        println!("{}", tracker.to_yaml());
    }
    let save_file = args.output.unwrap_or({
        let mut input_file = args.combat_file.clone();
        input_file.set_file_name(format!(
            "{}_save.yml",
            input_file.file_stem().unwrap().to_string_lossy()
        ));
        input_file
    });
    fs::write(save_file, tracker.to_yaml()).expect("Failed to write to file.");
    Ok(())
}
