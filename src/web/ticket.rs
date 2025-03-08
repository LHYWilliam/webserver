use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::{
    error::{DatabaseError, Result, TicketError},
    middleware::auth,
};

#[derive(Serialize)]
struct Ticket {
    pub id: i64,
    pub title: String,
}

pub fn router(pool: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/ticket", routing::post(create))
        .route("/ticket", routing::get(list))
        .route("/ticket", routing::delete(delete))
        .layer(middleware::from_fn_with_state(pool.clone(), auth::auth))
        .with_state(pool.clone())
}

#[derive(Deserialize)]
struct TitlePayload {
    pub title: String,
}

async fn create(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<TitlePayload>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handle post /ticket", "Handler");

    sqlx::query!(
        r#"
        insert into tickets (title)
        values (?)
        "#,
        payload.title,
    )
    .execute(&pool)
    .await
    .map_err(|_| DatabaseError::InsertFailed)?;

    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        select id, title
        from tickets
        order by id desc 
        limit 1;
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| TicketError::CreateFailed)?;

    Ok((StatusCode::CREATED, Json(ticket)))
}

async fn list(State(pool): State<Pool<Sqlite>>) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handle get /ticket", "Handler");

    let tickets = sqlx::query_as!(
        Ticket,
        r#"
        select id, title
        from tickets
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| DatabaseError::SelectFailed)?;

    Ok((StatusCode::OK, Json(tickets)))
}

#[derive(Deserialize)]
struct IdPayload {
    pub id: i64,
}

async fn delete(
    State(pool): State<Pool<Sqlite>>,
    Query(payload): Query<IdPayload>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handle delete /ticket", "Handler");

    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        select id, title
        from tickets
        where id = ?
        "#,
        payload.id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| TicketError::NotFound(payload.id))?;

    sqlx::query!(
        r#"
        delete from tickets
        where id = ?
        "#,
        payload.id,
    )
    .execute(&pool)
    .await
    .map_err(|_| DatabaseError::DeleteFailed)?;

    Ok((StatusCode::OK, Json(ticket)))
}
