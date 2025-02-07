use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::model::ticket::TicketController;
use crate::{error::Result, middleware::auth};

#[derive(Deserialize)]
struct TicketPayload {
    pub title: String,
}

#[derive(Deserialize)]
struct IdPayload {
    pub id: u64,
}

pub fn router(controller: TicketController) -> Router {
    axum::Router::new()
        .route("/ticket", post(create))
        .route("/ticket", get(list))
        .route("/ticket", axum::routing::delete(delete))
        .layer(middleware::from_fn(auth::auth))
        .with_state(controller)
}

async fn create(
    State(controller): State<TicketController>,
    Json(payload): Json<TicketPayload>,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle post /ticket", "Handler");

    let ticket = controller.create(payload.title).await?;

    Ok((StatusCode::CREATED, Json(ticket)))
}

async fn list(State(controller): State<TicketController>) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle get /ticket", "Handler");

    let tickets = controller.list().await?;

    Ok((StatusCode::OK, Json(tickets)))
}

async fn delete(
    State(controller): State<TicketController>,
    Query(id): Query<IdPayload>,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle delete /ticket", "Handler");

    let ticket = controller.delete(id.id).await?;

    Ok((StatusCode::OK, Json(ticket)))
}
