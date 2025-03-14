use std::sync::Arc;

use axum::{
    Extension,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use dashmap::{DashMap, DashSet};
use futures_util::{
    SinkExt,
    stream::{SplitSink, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tracing::{error, info, warn};

use super::{
    AppState,
    message::{ChannelMessage, SocketMessage},
};
use crate::middleware::jwt::Claims;

pub type Users = DashSet<Arc<User>>;
pub type Rooms = DashSet<Arc<Room>>;
pub type ConnectedUsers = DashSet<Arc<User>>;
pub type UserRooms = DashMap<Arc<User>, DashSet<Arc<Room>>>;
pub type RoomUsers = DashMap<Arc<Room>, DashSet<Arc<User>>>;

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub sender: Sender<Arc<ChannelMessage>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Room {
    pub name: String,
}

pub async fn chat(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let username = claims.sub;

        let Some(user) = state
            .users
            .iter()
            .find(|user| user.name == username)
            .map(|user| user.clone())
        else {
            error!("[{:^12}] ━ user {} not found", "WebSocket", username);
            return;
        };

        if state.connected_users.contains(&user) {
            error!(
                "[{:^12}] ━ user {} already connected",
                "WebSocket", user.name
            );
            return;
        };

        let (mut socket_tx, mut socket_rx) = socket.split();
        let mut channel_receiver = user.sender.subscribe();

        state.connected_users.insert(user.clone());
        info!("[{:^12}] ━ user {} connect", "WebSocket", user.name,);

        let user_clone = user.clone();
        let state_clone = state.clone();
        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = socket_rx.next().await {
                match message {
                    Message::Text(message) => {
                        socket_message_text_handler(
                            user_clone.clone(),
                            state_clone.clone(),
                            message.to_string(),
                        )
                        .await;
                    }

                    Message::Close(_) => {
                        socket_message_close_handler(user_clone.clone(), state_clone.clone()).await
                    }

                    _ => {}
                }
            }
        });

        let mut send_task = tokio::spawn(async move {
            while let Ok(message) = channel_receiver.recv().await {
                channel_message_handler(message, &mut socket_tx).await;
            }
        });

        tokio::select! {
            _ = &mut send_task => { receive_task.abort() },
            _ = &mut receive_task => { send_task.abort() },
        };

        state.connected_users.remove(&user);
        info!("[{:^12}] ━ user {} disconnect", "WebSocket", user.name);
    })
}

async fn socket_message_text_handler(user: Arc<User>, state: Arc<AppState>, message: String) {
    let Ok(message) = serde_json::from_str::<SocketMessage>(&message) else {
        error!("[{:^12}] ━ Invalid Message", "WebSocket");
        return;
    };

    let message = match message {
        SocketMessage::Join(name) => {
            let Some(room) = state
                .rooms
                .iter()
                .find(|room| room.name == name)
                .map(|room| room.clone())
            else {
                error!("[{:^12}] ━ room {} not found", "WebSocket", name);
                return;
            };

            if is_user_in_room(&state, &user, &room) {
                warn!(
                    "[{:^12}] ━ user {} already in room {}",
                    "WebSocket", user.name, room.name
                );
                return;
            };

            state.user_rooms.entry(user.clone()).and_modify(|rooms| {
                rooms.insert(room.clone());
            });
            state.room_users.entry(room.clone()).and_modify(|users| {
                users.insert(user.clone());
            });

            Arc::new(ChannelMessage {
                room: room.clone(),
                from: Some(user.name.clone()),
                message: format!("user {} join room {}", user.name, room.name),
            })
        }

        SocketMessage::Leave(name) => {
            let Some(room) = state
                .rooms
                .iter()
                .find(|room| room.name == name)
                .map(|room| room.clone())
            else {
                error!("[{:^12}] ━ room {} not found", "WebSocket", name);
                return;
            };

            if !is_user_in_room(&state, &user, &room) {
                warn!(
                    "[{:^12}] ━ user {} not in room {}",
                    "WebSocket", user.name, room.name
                );
                return;
            };

            state.user_rooms.entry(user.clone()).and_modify(|rooms| {
                rooms.remove(&room);
            });
            state.room_users.entry(room.clone()).and_modify(|users| {
                users.remove(&user);
            });

            Arc::new(ChannelMessage {
                room: room.clone(),
                from: Some(user.name.clone()),
                message: format!("user {} leave room {}", user.name, room.name),
            })
        }

        SocketMessage::Content(ChannelMessage { room, message, .. }) => {
            if !is_user_in_room(&state, &user, &room) {
                warn!(
                    "[{:^12}] ━ user {} not in room {}",
                    "WebSocket", user.name, room.name
                );
                return;
            };

            Arc::new(ChannelMessage {
                room,
                from: Some(user.name.clone()),
                message,
            })
        }
    };

    info!("[{:^12}] ━ {:?}", "WebSocket", message);

    state
        .room_users
        .entry(message.room.clone())
        .and_modify(|users| {
            users.iter().for_each(|user| {
                if let Err(e) = user.sender.send(message.clone()) {
                    error!("[{:^12}] ━ Send Message Error {}", "WebSocket", e);
                };
            })
        });
}

async fn socket_message_close_handler(user: Arc<User>, state: Arc<AppState>) {
    state.user_rooms.entry(user.clone()).and_modify(|rooms| {
        rooms.iter().for_each(|room| {
            let message = Arc::new(ChannelMessage {
                room: room.clone(),
                from: Some(user.name.clone()),
                message: format!("user {} leave room {}", user.name, room.name),
            });

            state.room_users.entry(room.clone()).and_modify(|users| {
                users.iter().for_each(|user| {
                    if let Err(e) = user.sender.send(message.clone()) {
                        error!("[{:^12}] ━ Send Message Error {}", "WebSocket", e);
                    };
                });
            });
        });
    });

    state.user_rooms.entry(user.clone()).and_modify(|rooms| {
        rooms.clear();
    });
    state.room_users.iter().for_each(|room| {
        room.value().remove(&user);
    });
}

async fn channel_message_handler(
    message: Arc<ChannelMessage>,
    tx: &mut SplitSink<WebSocket, Message>,
) {
    let Ok(message) = serde_json::to_string(&message) else {
        error!("[{:^12}] ━ Invalid Message", "WebSocket");
        return;
    };

    let message = Message::Text(message.into());

    if let Err(e) = tx.send(message).await {
        error!("[{:^12}] ━ Send Message Error {}", "WebSocket", e);
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for User {}

impl std::hash::Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

fn is_user_in_room(state: &AppState, user: &User, room: &Room) -> bool {
    state
        .user_rooms
        .get(user)
        .map(|rooms| rooms.contains(room))
        .unwrap_or(false)
}
