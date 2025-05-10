use color_eyre::Result;
use shared::{ClientMessage, ServerMessage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<()> {
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
