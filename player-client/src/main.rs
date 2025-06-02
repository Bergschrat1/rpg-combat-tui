use clap::Parser;
use color_eyre::Result;
use env_logger::{Builder, Target};
use log::{debug, info};
use std::fs::File;
use std::io::Write;

mod app;
mod cli;
mod terminal;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let log_file = File::create("player-client.log").expect("Failed to create log file");
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
    tokio::task::spawn_blocking(move || {
        let mut terminal = terminal::init().expect("failed to init terminal");
        let mut app = app::App::new(&args).expect("failed to start app");
        let _result = app.run(&mut terminal);

        if let Err(err) = terminal::restore() {
            eprintln!("failed to restore terminal: {err}");
        }
    })
    .await
    .expect("UI task panicked");

    Ok(())
}
