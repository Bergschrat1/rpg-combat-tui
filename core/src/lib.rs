use serde::{Deserialize, Serialize};

pub mod combat;
pub mod dto;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    GetPlayerView,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    DmView(String),
    PlayerView(String),
    CombatState(String),
}
