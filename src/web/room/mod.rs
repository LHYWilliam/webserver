mod chat;
mod manage;
mod message;

use std::sync::Arc;

use axum::{Router, middleware, routing};

use crate::middleware::jwt::Claims;
use chat::{RoomUsers, Rooms, UserRooms, Users};

#[derive(Default)]
struct AppState {
    users: Users,
    rooms: Rooms,
    room_users: RoomUsers,
    user_rooms: UserRooms,
}

pub fn router() -> Router {
    let state = Arc::new(AppState::default());

    Router::new()
        .route("/chat", routing::any(chat::chat))
        .route("/chat/user", routing::get(manage::list_user))
        .route("/chat/room_user", routing::delete(manage::delete_user))
        .route("/chat/room", routing::post(manage::create_room))
        .route("/chat/room", routing::get(manage::list_rooms))
        .route("/chat/room", routing::delete(manage::delete_room))
        .route("/chat/user_rooms", routing::get(manage::list_user_rooms))
        .route("/chat/room_users", routing::get(manage::list_room_users))
        .layer(middleware::from_extractor::<Claims>())
        .route("/chat/user", routing::post(manage::create_user))
        .with_state(state.clone())
}
