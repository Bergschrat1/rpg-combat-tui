use serde::{Deserialize, Serialize};

pub mod combat;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    GetPlayerView,
    GetDmView,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    DmView(String),
    PlayerView(String),
    CombatState(String),
}
