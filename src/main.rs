#![allow(dead_code)]

use log::info;
use std::fs::File;
use std::io::Write;

use clap::Parser;
use color_eyre::Result;
use env_logger::{Builder, Target};
mod combat;
mod tui;

use crate::tui::{app, cli, terminal};

fn main() -> Result<()> {
    color_eyre::install()?;
    let log_file = File::create("rpg_combat_tui.log").expect("Failed to create log file");
    Builder::from_default_env()
        .format_timestamp_secs()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:L{} [{}] - {}",
                record.file().unwrap_or("Unknown File"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    // Example log messages
    info!("Application started");

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
    // print to stdout if --stdout flag is given
    if args.stdout {
        println!("{}", tracker.to_yaml());
    }
    Ok(())
}
