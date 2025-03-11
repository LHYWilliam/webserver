mod chat;
mod manage;
mod message;

use std::sync::Arc;

use axum::{Router, routing};
use dashmap::DashMap;

use chat::{RoomUsers, UserRooms};

struct AppState {
    room_users: RoomUsers,
    user_rooms: UserRooms,
}

pub fn router() -> Router {
    let state = Arc::new(AppState {
        room_users: Arc::new(DashMap::new()),
        user_rooms: Arc::new(DashMap::new()),
    });

    Router::new()
        .route("/chat", routing::any(chat::chat))
        .route("/chat/user", routing::get(manage::list_room_users))
        .route("/chat/room", routing::get(manage::list_user_rooms))
        .route("/chat/room", routing::post(manage::create))
        .route("/chat/romm", routing::delete(manage::delete))
        .with_state(state)
}
