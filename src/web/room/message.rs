use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::chat::Room;

#[derive(Serialize, Deserialize, Debug)]
pub enum SocketMessage {
    Join(String),
    Leave(String),
    Content(ChannelMessage),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ChannelMessage {
    pub room: Room,
    pub from: Option<SocketAddr>,
    pub message: String,
}
