mod chat;
mod manage;
mod message;

use std::sync::Arc;

use axum::{Router, middleware, routing};

use crate::middleware::jwt::Claims;
use chat::{ConnectedUsers, RoomUsers, Rooms, UserRooms, Users};

#[derive(Default)]
struct AppState {
    users: Users,
    rooms: Rooms,
    connected_users: ConnectedUsers,
    user_rooms: UserRooms,
    room_users: RoomUsers,
}

pub fn router() -> Router {
    let state = Arc::new(AppState::default());

    Router::new()
        .route("/chat", routing::any(chat::chat))
        .route("/chat/user", routing::post(manage::create_user))
        .route("/chat/user", routing::get(manage::list_user))
        .route("/chat/user", routing::delete(manage::delete_user))
        .route("/chat/room", routing::post(manage::create_room))
        .route("/chat/room", routing::get(manage::list_rooms))
        .route("/chat/room", routing::delete(manage::delete_room))
        .route("/chat/user_rooms", routing::get(manage::list_user_rooms))
        .route("/chat/room_users", routing::get(manage::list_room_users))
        .route_layer(middleware::from_extractor::<Claims>())
        .with_state(state.clone())
}
