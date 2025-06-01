#![allow(dead_code)]

use core::combat::tracker::CombatTracker;
use log::info;
use std::io::Write;
use std::{fs::File, sync::Arc};
use tokio::sync::Mutex;
use tui::utils::{load_combat_yaml, validate_yaml_extension};

use clap::Parser;
use color_eyre::Result;
use env_logger::{Builder, Target};

mod tui;

use crate::tui::{app, cli, terminal};
use tui::server::run_server;

#[tokio::main]
async fn main() -> Result<()> {
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

    info!("Application started");

    let args = cli::Args::parse();
    validate_yaml_extension(&args.combat_file)?;
    let combat_yaml_string = load_combat_yaml(&args)?;
    info!(
        "Loaded YAML:
{}",
        &combat_yaml_string
    );

    let tracker = Arc::new(Mutex::new(CombatTracker::from_yaml(combat_yaml_string)));

    // Start server
    let server_tracker = Arc::clone(&tracker);
    tokio::spawn(async move {
        if let Err(e) = run_server(server_tracker).await {
            eprintln!("Server error: {e}");
        }
    });

    // Spawn blocking UI thread safely
    let ui_tracker = Arc::clone(&tracker);
    let ui_args = args.clone();
    let stdout = args.stdout;

    tokio::task::spawn_blocking(move || {
        let mut terminal = terminal::init().expect("failed to init terminal");
        let mut app =
            app::App::new_with_tracker(&ui_args, ui_tracker).expect("failed to start app");
        let result = app.run(&mut terminal);

        if let Err(err) = terminal::restore() {
            eprintln!("failed to restore terminal: {err}");
        }

        if stdout {
            if let Ok(tracker) = result {
                println!("{}", tracker.to_yaml());
            }
        }
    })
    .await
    .expect("UI task panicked");

    Ok(())
}
