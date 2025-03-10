mod manage;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing,
};
use dashmap::{DashMap, DashSet};
use futures_util::{
    SinkExt,
    stream::{SplitSink, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender};

use crate::error::Result;

pub fn router() -> Router {
    let state = Arc::new(AppState {
        room_users: Arc::new(DashMap::new()),
        _user_rooms: Arc::new(DashMap::new()),
    });

    Router::new()
        .route("/chat", routing::any(chat))
        .route("/chat/manage", routing::get(manage::manage_get))
        .route("/chat/manage", routing::post(manage::manage_post))
        .route("/chat/manage", routing::delete(manage::manage_delete))
        .with_state(state)
}

#[derive(Serialize, Deserialize, Debug)]
enum SocketMessage {
    Join(String),
    Leave(String),
    Content(ChannelMessage),
}

struct User {
    name: String,
    addr: SocketAddr,
    sender: Sender<Arc<ChannelMessage>>,
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

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
struct Room {
    name: String,
}
type RoomUsers = Arc<DashMap<Room, DashSet<User>>>;
type UserRooms = Arc<DashMap<User, DashSet<Room>>>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct ChannelMessage {
    room: Room,
    from: Option<SocketAddr>,
    message: String,
}

struct AppState {
    room_users: RoomUsers,
    _user_rooms: UserRooms,
}

async fn chat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - socket {} connect", "WebSocket", addr);

    Ok(ws.on_upgrade(move |socket| async move {
        let (mut socket_tx, mut socket_rx) = socket.split();
        let (channel_sender, mut channel_receiver) = broadcast::channel::<Arc<ChannelMessage>>(128);

        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = socket_rx.next().await {
                socket_message_handler(
                    message,
                    channel_sender.clone(),
                    addr,
                    state.room_users.clone(),
                )
                .await;
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
    }))
}

async fn socket_message_handler(
    message: Message,
    sender: Sender<Arc<ChannelMessage>>,
    addr: SocketAddr,
    room_users: RoomUsers,
) {
    let message = match message {
        Message::Text(text) => text.to_string(),
        Message::Close(_) => serde_json::to_string(&SocketMessage::Leave("1".into())).unwrap(),
        _ => serde_json::to_string(&SocketMessage::Content(ChannelMessage {
            room: Room { name: "1".into() },
            from: Some(addr),
            message: "invalid message".to_string(),
        }))
        .unwrap(),
    };

    let message =
        serde_json::from_str::<SocketMessage>(&message).unwrap_or(SocketMessage::Leave("1".into()));

    let message = match message {
        SocketMessage::Join(name) => {
            room_users
                .get(&Room { name: name.clone() })
                .unwrap()
                .insert(User {
                    name: addr.to_string(),
                    addr,
                    sender,
                });

            Arc::new(ChannelMessage {
                room: Room { name },
                from: Some(addr),
                message: format!("{addr} join"),
            })
        }
        SocketMessage::Leave(name) => {
            room_users
                .get(&Room { name: name.clone() })
                .unwrap()
                .remove(&User {
                    name: addr.to_string(),
                    addr,
                    sender,
                });

            Arc::new(ChannelMessage {
                room: Room { name },
                from: Some(addr),
                message: format!("{addr} leave"),
            })
        }
        SocketMessage::Content(ChannelMessage { room, message, .. }) => Arc::new(ChannelMessage {
            room,
            from: Some(addr),
            message,
        }),
    };

    room_users
        .get(&message.room)
        .unwrap()
        .iter()
        .for_each(|user| {
            user.sender.send(message.clone()).unwrap();
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
