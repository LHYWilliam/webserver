use std::net::SocketAddr;

use axum::{
    Router,
    extract::{ConnectInfo, WebSocketUpgrade},
    response::IntoResponse,
    routing,
};
use futures_util::{SinkExt, stream::StreamExt};

use crate::error::Result;

pub fn router() -> Router {
    Router::new().route("/chat", routing::any(chat))
}

async fn chat(ws: WebSocketUpgrade, info: ConnectInfo<SocketAddr>) -> Result<impl IntoResponse> {
    println!(
        "[{:^12}] - socket {}:{} connect",
        "WebSocket",
        info.ip(),
        info.port()
    );

    Ok(ws.on_upgrade(move |socket| async move {
        let (mut tx, mut rx) = socket.split();

        while let Some(Ok(msg)) = rx.next().await {
            if tx.send(msg).await.is_err() {
                println!(
                    "[{:^12}] - socket {}:{} disconnect\n",
                    "WebSocket",
                    info.ip(),
                    info.port()
                );
                break;
            };
        }
    }))
}
