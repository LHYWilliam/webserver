use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, State, WebSocketUpgrade, ws},
    response::IntoResponse,
    routing,
};
use dashmap::DashMap;
use futures_util::{SinkExt, stream::StreamExt};
use tokio::sync::broadcast::{Sender, channel};

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
        let (sender, mut receiver) = channel::<Arc<Message>>(128);

        users.insert(addr, sender);

        let users = users.clone();
        tokio::spawn(async move {
            while let Some(Ok(message)) = rx.next().await {
                let message = match message {
                    ws::Message::Text(text) => text.to_string(),
                    _ => continue,
                };

                let message = Arc::new(Message::Content(message));

                users.iter().for_each(|user| {
                    user.value().send(message.clone()).unwrap();
                });
            }
        });

        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                let message = match &*message {
                    Message::Content(content) => ws::Message::Text(content.into()),
                };

                tx.send(message).await.unwrap();
            }
        });
    }))
}
