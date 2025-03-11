use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashSet;
use serde::Deserialize;

use super::{AppState, chat::Room};
use crate::error::{DatabaseError, Result};

pub async fn list_room_users(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handl get /chat/user", "Handler");

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

    Ok(Json(room_users))
}

pub async fn list_user_rooms(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handl get /chat/room", "Handler");

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

    Ok(Json(user_rooms))
}

#[derive(Deserialize)]
pub struct PostPayload {
    name: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PostPayload>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handl post /chat/manage", "Handler");

    state.room_users.insert(
        Room {
            name: payload.name.clone(),
        },
        DashSet::new(),
    );

    Ok((StatusCode::CREATED, Json(Room { name: payload.name })))
}

#[derive(Deserialize)]
pub struct DeletePayload {
    name: String,
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<DeletePayload>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handl delete /chat/manage", "Handler");

    state
        .room_users
        .remove(&Room {
            name: payload.name.clone(),
        })
        .ok_or(DatabaseError::DeleteFailed)?;

    Ok((StatusCode::OK, Json(Room { name: payload.name })))
}
