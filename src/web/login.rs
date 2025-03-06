use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tower_cookies::{Cookie, Cookies};

use crate::{
    error::{Error, Result},
    middleware::auth::jwt::{AuthBody, Claims},
};

pub fn router(pool: Pool<Sqlite>) -> Router {
    Router::new().route("/login", routing::post(login).with_state(pool))
}

#[derive(Serialize, Deserialize)]
struct LoginPayload {
    pub username: String,
    pub password: String,
}

async fn login(
    cookies: Cookies,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - handle post /login", "Handler");

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
    .map_err(|_| Error::AuthErrorWrongUsernameOrPassword)?;

    cookies.add(Cookie::new("user".to_string(), payload.username.clone()));

    let claims = Claims {
        sub: payload.username.clone(),
        exp: (Utc::now() + Duration::minutes(1)).timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret"),
    )
    .map_err(|_| Error::AuthErrorInvalidToken)?;

    Ok((StatusCode::OK, Json(AuthBody::new(token))))
}
