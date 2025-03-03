use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing,
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::error::{Error, Result};

#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
    password: String,
}

pub fn router(pool: Pool<Sqlite>) -> Router {
    axum::Router::new()
        .route("/register", routing::post(register))
        .with_state(pool)
}

async fn register(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle post /register", "Handler");

    let result = sqlx::query!(
        r#"
        insert into users (username, password)
        values (?, ?)
        on conflict (username) do nothing
        "#,
        payload.username,
        payload.password,
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::SQLiteErrorRegisterFailed {
        username: payload.username.clone(),
    })?;

    match result.rows_affected() {
        0 => Ok((
            StatusCode::CONFLICT,
            Html(format!("User {} already exists", payload.username)),
        )),
        _ => Ok((
            StatusCode::CREATED,
            Html(format!("User {} created", payload.username)),
        )),
    }
}
