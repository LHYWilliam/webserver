use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, State, WebSocketUpgrade, ws},
    response::IntoResponse,
    routing,
};
use dashmap::DashMap;
use futures_util::{SinkExt, stream::StreamExt};
use tokio::sync::broadcast::{self, Sender};

use crate::error::Result;

#[derive(Debug)]
enum Message {
    Content(String),
}

type Users = DashMap<SocketAddr, Sender<Arc<Message>>>;

pub fn router() -> Router {
    let users = Arc::new(Users::new());

    Router::new()
        .route("/chat", routing::any(chat))
        .with_state(users)
}

async fn chat(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(users): State<Arc<Users>>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - socket {} connect", "WebSocket", addr);

    Ok(ws.on_upgrade(move |socket| async move {
        let (mut tx, mut rx) = socket.split();
        let (sender, mut receiver) = broadcast::channel::<Arc<Message>>(128);

        users.insert(addr, sender);

        let mut send_task = tokio::spawn(async move {
            while let Some(Ok(message)) = rx.next().await {
                match message {
                    ws::Message::Text(text) => {
                        let message = Arc::new(Message::Content(text.to_string()));

                        users.iter().for_each(|user| {
                            user.value().send(message.clone()).unwrap();
                        });
                    }
                    ws::Message::Close(_) => {
                        users.remove(&addr);
                        break;
                    }
                    _ => continue,
                };
            }
        });

        let mut receive_tasek = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                match &*message {
                    Message::Content(content) => {
                        let message = ws::Message::Text(content.into());

                        tx.send(message).await.unwrap();
                    }
                };
            }
        });

        tokio::select! {
            _ = &mut send_task => {
                println!("[{:^12}] - socket {} disconnect\n", "WebSocket", addr);
                receive_tasek.abort()
            },

            _ = &mut receive_tasek => {
                println!("[{:^12}] - socket {} disconnect\n", "WebSocket", addr);
                send_task.abort()
            },
        }
    }))
}
