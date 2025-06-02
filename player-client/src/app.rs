use core::dto::PlayerClientState;
use core::ClientMessage;
use core::ServerMessage;

use crate::terminal;
use crate::ui::TableColors;
use crate::{cli::Args, ui};
use color_eyre::Result;
use log::debug;
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct App {
    pub exit: bool,
    pub colors: TableColors,
    pub tracker: PlayerClientState,
}

impl App {
    pub fn new(args: &Args) -> Result<Self> {
        info!("Initializing App");
        Ok(Self {
            exit: false,
            colors: TableColors::new(),
            tracker: PlayerClientState::default(),
        })
    }

    pub async fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        debug!("Running program.");
        let mut stream = TcpStream::connect("127.0.0.1:8000").await?;
        let request = ClientMessage::GetPlayerView;
        let json = serde_json::to_vec(&request)?;
        while !self.exit {
            // Send a player view request
            stream.write_all(&json).await?;
            // Wait for and read the response
            let mut buf = vec![];
            let n = stream.read_to_end(&mut buf).await?;
            let response: ServerMessage = serde_json::from_slice(&buf[..n])?;
            let player_state: PlayerClientState = match response {
                ServerMessage::PlayerView(data) => serde_json::from_str(&data)?,
                ServerMessage::DmView(_) => todo!(),
                ServerMessage::CombatState(_) => todo!(),
            };
            self.tracker = player_state;

            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
        }
        Ok(())
    }
}
