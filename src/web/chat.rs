use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{self, Message},
    },
    response::IntoResponse,
    routing,
};
use dashmap::DashMap;
use futures_util::{
    SinkExt,
    stream::{SplitSink, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender};

use crate::error::Result;

type ChannelSender = Sender<Arc<ChannelMessage>>;
type Users = DashMap<SocketAddr, ChannelSender>;
type RoomUsers = Arc<Users>;

#[derive(Serialize, Deserialize, Debug)]
enum SocketMessage {
    Join,
    Leave,
    Content(ChannelMessage),
}

#[derive(Serialize, Deserialize, Debug)]
struct ChannelMessage {
    from: Option<SocketAddr>,
    message: String,
}

struct AppState {
    room_users: RoomUsers,
}

pub fn router() -> Router {
    let state = Arc::new(AppState {
        room_users: Arc::new(Users::new()),
    });

    Router::new()
        .route("/chat", routing::any(chat))
        .with_state(state)
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
                socket_message_handler(message, channel_sender.clone(), addr, &state.room_users)
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
    sender: ChannelSender,
    addr: SocketAddr,
    room_users: &RoomUsers,
) {
    let message = match message {
        ws::Message::Text(text) => text.to_string(),
        ws::Message::Close(_) => serde_json::to_string(&SocketMessage::Leave).unwrap(),
        _ => serde_json::to_string(&SocketMessage::Content(ChannelMessage {
            from: Some(addr),
            message: "invalid message".to_string(),
        }))
        .unwrap(),
    };

    let message = serde_json::from_str::<SocketMessage>(&message).unwrap_or(SocketMessage::Leave);

    let message = match message {
        SocketMessage::Join => {
            room_users.insert(addr, sender);
            format!("{addr} join")
        }
        SocketMessage::Leave => {
            room_users.remove(&addr);
            format!("{addr} leave")
        }
        SocketMessage::Content(ChannelMessage { message, .. }) => message,
    };

    let message = Arc::new(ChannelMessage {
        from: Some(addr),
        message,
    });

    room_users.iter().for_each(|user| {
        user.value().send(message.clone()).unwrap();
    });
}

async fn channel_message_handler(
    message: Arc<ChannelMessage>,
    tx: &mut SplitSink<ws::WebSocket, ws::Message>,
) {
    let message = serde_json::to_string(&message).unwrap();
    let message = ws::Message::Text(message.into());

    tx.send(message).await.unwrap();
}
