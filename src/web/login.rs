use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use tower_cookies::{Cookie, Cookies};

use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, FromRow, PartialEq)]
struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub fn router(pool: Pool<Sqlite>) -> Router {
    Router::new().route("/login", routing::post(login).with_state(pool))
}

async fn login(
    cookies: Cookies,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle post /login", "Handler");

    sqlx::query!(
        r#"
        select *
        from users
        where username = ?
        and password = ?
        "#,
        payload.username,
        payload.password,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::AuthErrorWorngUsernameOrPassword)?;

    cookies.add(Cookie::new(
        "username".to_string(),
        payload.username.clone(),
    ));
    cookies.add(Cookie::new(
        "password".to_string(),
        payload.password.clone(),
    ));

    Ok((StatusCode::OK, Json(payload)))
}
