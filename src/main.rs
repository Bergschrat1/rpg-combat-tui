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
    if let Some(path) = args.output {
        fs::write(path, tracker.to_yaml()).expect("Failed to write to file.");
    }
    Ok(())
}
