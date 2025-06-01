use clap::Parser;
use color_eyre::Result;
use env_logger::{Builder, Target};
use log::info;
use core::{ClientMessage, ServerMessage};
use std::fs::File;
use std::io::Write;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod app;
mod cli;
mod player_state;
mod terminal;
mod ui;

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
    tokio::task::spawn_blocking(move || {
        let mut terminal = terminal::init().expect("failed to init terminal");
        let mut app = app::App::new(&args).expect("failed to start app");
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

    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;

    // Send a player view request
    let request = ClientMessage::GetPlayerView;
    let json = serde_json::to_vec(&request)?;
    stream.write_all(&json).await?;

    // Wait for and read the response
    let mut buf = vec![];
    let n = stream.read_to_end(&mut buf).await?;
    let response: ServerMessage = serde_json::from_slice(&buf[..n])?;

    match response {
        ServerMessage::PlayerView(data) => println!("Player View: {}", data),
        ServerMessage::DmView(_) => println!("Unexpected DM view for player."),
        ServerMessage::CombatState(_) => todo!(),
    }

    Ok(())
}
