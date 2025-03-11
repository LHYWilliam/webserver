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
use tokio::sync::broadcast::{self, Sender};

use super::{
    AppState,
    message::{ChannelMessage, SocketMessage},
};

pub type RoomUsers = Arc<DashMap<Room, DashSet<User>>>;
pub type UserRooms = Arc<DashMap<User, DashSet<Room>>>;

#[derive(Clone)]
pub struct User {
    pub name: String,
    addr: SocketAddr,
    sender: Sender<Arc<ChannelMessage>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Room {
    pub name: String,
}

pub async fn chat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("[{:^12}] - socket {} connect", "WebSocket", addr);

    ws.on_upgrade(move |socket| async move {
        let (mut socket_tx, mut socket_rx) = socket.split();
        let (channel_sender, mut channel_receiver) = broadcast::channel::<Arc<ChannelMessage>>(128);

        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = socket_rx.next().await {
                match message {
                    Message::Text(message) => {
                        socket_message_text_handler(
                            message.to_string(),
                            channel_sender.clone(),
                            addr,
                            state.room_users.clone(),
                            state.user_rooms.clone(),
                        )
                        .await;
                    }

                    Message::Close(_) => {
                        socket_message_close_handler(
                            channel_sender.clone(),
                            addr,
                            state.room_users.clone(),
                            state.user_rooms.clone(),
                        )
                        .await
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

async fn socket_message_text_handler(
    message: String,
    sender: Sender<Arc<ChannelMessage>>,
    addr: SocketAddr,
    room_users: RoomUsers,
    user_rooms: UserRooms,
) {
    let Ok(message) = serde_json::from_str::<SocketMessage>(&message) else {
        return;
    };

    let user = User {
        name: addr.to_string(),
        addr,
        sender: sender.clone(),
    };

    let message = match message {
        SocketMessage::Join(name) => {
            let room = Room { name };

            if !room_users.contains_key(&room) {
                return;
            }

            user_rooms
                .entry(user.clone())
                .or_default()
                .insert(room.clone());
            room_users.entry(room.clone()).and_modify(|users| {
                users.insert(user);
            });

            Arc::new(ChannelMessage {
                room,
                from: Some(addr),
                message: format!("{addr} join"),
            })
        }

        SocketMessage::Leave(name) => {
            let room = Room { name };

            user_rooms.entry(user.clone()).and_modify(|rooms| {
                rooms.remove(&room);
            });
            room_users.entry(room.clone()).and_modify(|users| {
                users.remove(&user);
            });

            Arc::new(ChannelMessage {
                room: room.clone(),
                from: Some(addr),
                message: format!("user {} leave leave room {}", addr, room.name),
            })
        }

        SocketMessage::Content(ChannelMessage { room, message, .. }) => Arc::new(ChannelMessage {
            room,
            from: Some(addr),
            message,
        }),
    };

    room_users.entry(message.room.clone()).and_modify(|users| {
        users.iter().for_each(|user| {
            user.sender.send(message.clone()).unwrap();
        })
    });
}

async fn socket_message_close_handler(
    sender: Sender<Arc<ChannelMessage>>,
    addr: SocketAddr,
    room_users: RoomUsers,
    user_rooms: UserRooms,
) {
    let user = User {
        name: addr.to_string(),
        addr,
        sender: sender.clone(),
    };

    user_rooms.entry(user.clone()).and_modify(|rooms| {
        rooms.iter().for_each(|room| {
            let message = Arc::new(ChannelMessage {
                room: room.clone(),
                from: Some(addr),
                message: format!("user {} leave room {}", addr, room.name),
            });

            room_users.entry(room.clone()).and_modify(|users| {
                users.iter().for_each(|user| {
                    user.sender.send(message.clone()).unwrap();
                });
            });
        });

        user_rooms.remove(&user);
        room_users.iter().for_each(|room| {
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
