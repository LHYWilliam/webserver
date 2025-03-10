mod chat;
mod manage;
mod message;

use std::sync::Arc;

use axum::{Router, routing};
use dashmap::DashMap;

use chat::{RoomUsers, UserRooms};

struct AppState {
    room_users: RoomUsers,
    _user_rooms: UserRooms,
}

pub fn router() -> Router {
    let state = Arc::new(AppState {
        room_users: Arc::new(DashMap::new()),
        _user_rooms: Arc::new(DashMap::new()),
    });

    Router::new()
        .route("/chat", routing::any(chat::chat))
        .route("/chat/manage", routing::get(manage::list))
        .route("/chat/manage", routing::post(manage::create))
        .route("/chat/manage", routing::delete(manage::delete))
        .with_state(state)
}
