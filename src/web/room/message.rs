use std::sync::Arc;

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
    pub room: Arc<Room>,
    pub from: Option<String>,
    pub message: String,
}
