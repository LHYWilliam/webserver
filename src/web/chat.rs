use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, State, WebSocketUpgrade, ws},
    response::IntoResponse,
    routing,
};
use dashmap::DashMap;
use futures_util::{
    SinkExt,
    stream::{SplitSink, StreamExt},
};
use serde::Deserialize;
use tokio::sync::broadcast::{self, Sender};

use crate::error::Result;

#[derive(Deserialize, Debug)]
enum Message {
    Join,
    Leave,
    Content(String),
}

type Users = DashMap<SocketAddr, Sender<Arc<Message>>>;

struct AppState {
    room_users: Arc<Users>,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - socket {} connect", "WebSocket", addr);

    Ok(ws.on_upgrade(move |socket| async move {
        let (mut tx, mut rx) = socket.split();
        let (sender, mut receiver) = broadcast::channel::<Arc<Message>>(128);

        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = rx.next().await {
                match message {
                    ws::Message::Text(text) => {
                        receive_handler(text.to_string(), &state.room_users, sender.clone(), addr)
                            .await;
                    }
                    ws::Message::Close(_) => {
                        state.room_users.remove(&addr);
                        break;
                    }
                    _ => continue,
                }
            }
        });

        let mut send_task = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                send_handler(message.clone(), &mut tx).await;
            }
        });

        tokio::select! {
            _ = &mut send_task => { receive_task.abort() },
            _ = &mut receive_task => { send_task.abort() },
        };

        println!("[{:^12}] - socket {} disconnect\n", "WebSocket", addr);
    }))
}

async fn receive_handler(
    message: String,
    room_users: &Users,
    sender: Sender<Arc<Message>>,
    addr: SocketAddr,
) {
    let message = serde_json::from_str::<Message>(&message).unwrap();

    match message {
        Message::Join => {
            room_users.insert(addr, sender);
        }

        Message::Leave => {
            room_users.remove(&addr);
        }

        Message::Content(message) => {
            let message = Arc::new(Message::Content(message));

            room_users.iter().for_each(|user| {
                user.value().send(message.clone()).unwrap();
            });
        }
    }
}

async fn send_handler(message: Arc<Message>, tx: &mut SplitSink<ws::WebSocket, ws::Message>) {
    match &*message {
        Message::Content(content) => {
            let message = ws::Message::Text(content.into());

            tx.send(message).await.unwrap();
        }
        Message::Join => {}
        Message::Leave => {}
    }
}
