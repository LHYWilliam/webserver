use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
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
use tower_cookies::Cookies;

use super::{
    AppState,
    message::{ChannelMessage, SocketMessage},
};

pub type Users = Arc<DashSet<Arc<User>>>;
pub type Rooms = Arc<DashSet<Arc<Room>>>;
pub type UserRooms = Arc<DashMap<Arc<User>, DashSet<Arc<Room>>>>;
pub type RoomUsers = Arc<DashMap<Arc<Room>, DashSet<Arc<User>>>>;

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub addr: String,
    pub sender: Sender<Arc<ChannelMessage>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Room {
    pub name: String,
}

pub async fn chat(
    ws: WebSocketUpgrade,
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let username = cookies.get("user").unwrap().value().to_string();
        let user = state
            .users
            .iter()
            .find(|user| user.name == username)
            .unwrap()
            .clone();

        let (mut socket_tx, mut socket_rx) = socket.split();
        let mut channel_receiver = user.sender.subscribe();

        println!(
            "[{:^12}] - socket {} {} connect",
            "WebSocket", user.name, user.addr
        );

        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = socket_rx.next().await {
                match message {
                    Message::Text(message) => {
                        socket_message_text_handler(
                            user.clone(),
                            message.to_string(),
                            state.clone(),
                        )
                        .await;
                    }

                    Message::Close(_) => {
                        socket_message_close_handler(user.clone(), state.clone()).await
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

        println!("[{:^12}] - socket {} disconnect\n", "WebSocket", addr);
    })
}

async fn socket_message_text_handler(user: Arc<User>, message: String, state: Arc<AppState>) {
    let Ok(message) = serde_json::from_str::<SocketMessage>(&message) else {
        return;
    };

    let message = match message {
        SocketMessage::Join(name) => {
            let room = state
                .rooms
                .iter()
                .find(|room| room.name == name)
                .unwrap()
                .clone();

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
            let room = state
                .rooms
                .iter()
                .find(|room| room.name == name)
                .unwrap()
                .clone();

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

        SocketMessage::Content(ChannelMessage { room, message, .. }) => Arc::new(ChannelMessage {
            room,
            from: Some(user.name.clone()),
            message,
        }),
    };

    state
        .room_users
        .entry(message.room.clone())
        .and_modify(|users| {
            users.iter().for_each(|user| {
                user.sender.send(message.clone()).unwrap();
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
                    user.sender.send(message.clone()).unwrap();
                });
            });
        });

        state.user_rooms.remove(&user);
        state.room_users.iter().for_each(|room| {
            room.value().remove(&user);
        });
    });
}

async fn channel_message_handler(
    message: Arc<ChannelMessage>,
    tx: &mut SplitSink<WebSocket, Message>,
) {
    let message = serde_json::to_string(&message).unwrap();
    let message = Message::Text(message.into());

    tx.send(message).await.unwrap();
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.addr == other.addr
    }
}

impl Eq for User {}

impl std::hash::Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.addr.hash(state);
    }
}
