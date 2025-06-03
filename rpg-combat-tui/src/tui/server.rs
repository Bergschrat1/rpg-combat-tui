use core::dto::PlayerClientState;
use std::sync::Arc;

use color_eyre::Result;
use core::{ClientMessage, ServerMessage};
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use core::combat::tracker::CombatTracker;

pub async fn run_server(state: Arc<Mutex<CombatTracker>>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Server listening on 127.0.0.1:8000");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            loop {
                let mut len_buf = [0u8; 4];
                if socket.read_exact(&mut len_buf).await.is_err() {
                    break; // connection closed or failed
                }
                let len = u32::from_be_bytes(len_buf) as usize;

                let mut buffer = vec![0u8; len];
                if socket.read_exact(&mut buffer).await.is_err() {
                    break;
                }

                match serde_json::from_slice::<ClientMessage>(&buffer) {
                    Ok(msg) => {
                        info!("Request received: {:?}", &msg);
                        let tracker = state.lock().await;
                        let dto_state = PlayerClientState::from(&*tracker);
                        let response = match msg {
                            ClientMessage::GetPlayerView => ServerMessage::PlayerView(
                                serde_json::to_string(&dto_state).unwrap(),
                            ),
                        };
                        info!("Sending response: {:?}", &response);
                        let json = serde_json::to_vec(&response).unwrap();
                        let len = (json.len() as u32).to_be_bytes();
                        if socket.write_all(&len).await.is_err() {
                            break;
                        }
                        if socket.write_all(&json).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Invalid message: {e}");
                        break;
                    }
                }
            }
        });
    }
}
