use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashSet;
use serde::Deserialize;

use crate::{
    error::{DatabaseError, Result},
    web::chat::{AppState, Room},
};

pub async fn manage_get(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handl get /chat/manage", "Handler");

    let rooms = state
        .room_users
        .iter()
        .map(|entry| entry.key().name.clone())
        .collect::<Vec<String>>();

    Ok(Json(rooms))
}

#[derive(Deserialize)]
pub struct PostPayload {
    name: String,
}

pub async fn manage_post(
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

pub async fn manage_delete(
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
