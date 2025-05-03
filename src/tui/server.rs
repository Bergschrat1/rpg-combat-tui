use color_eyre::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::shared::lib::{ClientMessage, ServerMessage};
pub async fn run_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Server listening on 127.0.0.1:8000");

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 1024];
            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => match serde_json::from_slice::<ClientMessage>(&buffer[..n]) {
                    Ok(msg) => {
                        let response = match msg {
                            ClientMessage::GetPlayerView => {
                                ServerMessage::PlayerView("HP: 18/30".into())
                            }
                            ClientMessage::GetDmView => {
                                ServerMessage::DmView("Goblin: HP 12, Orc: HP 45".into())
                            }
                        };
                        let json = serde_json::to_vec(&response).unwrap();
                        let _ = socket.write_all(&json).await;
                    }
                    Err(e) => eprintln!("Invalid JSON from client: {e}"),
                },
                Ok(_) => println!("Client closed connection without sending data."),
                Err(e) => eprintln!("Failed to read from socket: {e}"),
            }
        });
    }
}
