use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashSet;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use tracing::info;

use super::{
    AppState,
    chat::{Room, User},
    message::ChannelMessage,
};
use crate::error::{Result, RoomError};

#[derive(Deserialize)]
pub struct CreateUserPayload {
    name: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl post /chat/user", "Handler");

    let (sender, _) = broadcast::channel::<Arc<ChannelMessage>>(128);

    let user = Arc::new(User {
        name: payload.name,
        addr: addr.to_string(),
        sender,
    });

    state.users.insert(user.clone());
    state.user_rooms.insert(user.clone(), DashSet::new());

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "name": user.name,
            "addr": user.addr,
        })),
    ))
}

pub async fn list_user(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl get /chat/user", "Handler");

    let users = state
        .users
        .iter()
        .map(|user| user.name.clone())
        .collect::<Vec<String>>();

    Ok((StatusCode::OK, Json(users)))
}

#[derive(Deserialize)]
pub struct DeleteUserPayload {
    name: String,
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(payload): Query<DeleteUserPayload>,
) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl delete /chat/user", "Handler");

    let user = state
        .users
        .iter()
        .find(|user| user.name == payload.name && user.addr == addr.to_string())
        .ok_or(RoomError::UserNotFound)?
        .clone();

    state.users.remove(&user).ok_or(RoomError::UserNotFound)?;
    state
        .user_rooms
        .remove(&user)
        .ok_or(RoomError::UserNotFound)?;
    state.room_users.iter().for_each(|entry| {
        entry.value().remove(&user);
    });

    Ok((
        StatusCode::OK,
        Json(json!({ "name": user.name, "addr": user.addr })),
    ))
}

#[derive(Deserialize)]
pub struct CreateRoomPayload {
    name: String,
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRoomPayload>,
) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl post /chat/room", "Handler");

    let room = Arc::new(Room { name: payload.name });

    state.rooms.insert(room.clone());
    state.room_users.insert(room.clone(), DashSet::new());

    Ok((StatusCode::CREATED, Json(room)))
}

pub async fn list_rooms(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl get /chat/room", "Handler");

    let rooms = state
        .rooms
        .iter()
        .map(|room| room.name.clone())
        .collect::<Vec<String>>();

    Ok((StatusCode::OK, Json(rooms)))
}

#[derive(Deserialize)]
pub struct DeleteRoomPayload {
    name: String,
}

pub async fn delete_room(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<DeleteRoomPayload>,
) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl delete /chat/room", "Handler");

    let room = state
        .rooms
        .iter()
        .find(|room| room.name == payload.name)
        .ok_or(RoomError::RoomNotFound)?
        .clone();

    state.rooms.remove(&room).ok_or(RoomError::RoomNotFound)?;
    state
        .room_users
        .remove(&room)
        .ok_or(RoomError::RoomNotFound)?;
    state.user_rooms.iter().for_each(|entry| {
        entry.value().remove(&room);
    });

    Ok((StatusCode::OK, Json(room)))
}

pub async fn list_user_rooms(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl get /chat/user_rooms", "Handler");

    let user_rooms = state
        .user_rooms
        .iter()
        .map(|entry| {
            (
                entry.key().name.clone(),
                entry
                    .value()
                    .iter()
                    .map(|room| room.name.clone())
                    .collect::<Vec<String>>(),
            )
        })
        .collect::<HashMap<String, Vec<String>>>();

    Ok((StatusCode::OK, Json(user_rooms)))
}

pub async fn list_room_users(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    info!("[{:^12}] - handl get /chat/room_users", "Handler");

    let room_users = state
        .room_users
        .iter()
        .map(|entry| {
            (
                entry.key().name.clone(),
                entry
                    .value()
                    .iter()
                    .map(|user| user.name.clone())
                    .collect::<Vec<String>>(),
            )
        })
        .collect::<HashMap<String, Vec<String>>>();

    Ok((StatusCode::OK, Json(room_users)))
}
