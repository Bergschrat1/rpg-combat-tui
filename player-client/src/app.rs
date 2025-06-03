use core::dto::PlayerClientState;
use std::sync::Arc;

use color_eyre::Result;
use core::{ClientMessage, ServerMessage};
use log::{debug, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::terminal;
use crate::ui::TableColors;
use crate::{cli::Args, ui};

pub struct App {
    pub exit: bool,
    pub colors: TableColors,
    pub tracker: PlayerClientState,
}

impl App {
    pub fn new(_args: &Args) -> Result<Self> {
        info!("Initializing App");
        Ok(Self {
            exit: false,
            colors: TableColors::new(),
            tracker: PlayerClientState::default(),
        })
    }

    pub async fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        debug!("Running player client loop");
        let mut stream = TcpStream::connect("127.0.0.1:8000").await?;

        while !self.exit {
            // Send request with length prefix
            let request = ClientMessage::GetPlayerView;
            let json = serde_json::to_vec(&request)?;
            let len = (json.len() as u32).to_be_bytes();
            stream.write_all(&len).await?;
            stream.write_all(&json).await?;

            // Read response length prefix
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await?;
            let len = u32::from_be_bytes(len_buf) as usize;

            // Read response body
            let mut buf = vec![0u8; len];
            stream.read_exact(&mut buf).await?;
            let response: ServerMessage = serde_json::from_slice(&buf)?;

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
