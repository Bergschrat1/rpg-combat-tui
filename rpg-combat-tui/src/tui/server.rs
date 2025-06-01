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
            let mut buffer = vec![0u8; 1024];
            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => match serde_json::from_slice::<ClientMessage>(&buffer[..n]) {
                    Ok(msg) => {
                        info!("Request recieved: {:?}", &msg);
                        let response = {
                            let state = state.lock().await;
                            match msg {
                                ClientMessage::GetPlayerView => {
                                    ServerMessage::PlayerView(state.to_json())
                                }
                                ClientMessage::GetDmView => ServerMessage::DmView(state.to_json()),
                            }
                        };
                        info!("Sending response: {:?}", &response);
                        let json = serde_json::to_vec(&response).unwrap();
                        let _ = socket.write_all(&json).await;
                    }
                    Err(e) => {
                        eprintln!("Invalid JSON from client: {e}");
                    }
                },
                Ok(_) => println!("Client closed connection without sending data."),
                Err(e) => eprintln!("Failed to read from socket: {e}"),
            }
        });
    }
}
