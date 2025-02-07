use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

#[derive(Serialize, Deserialize)]
struct LoginPayload {
    pub username: String,
    pub password: String,
}

use crate::error::Result;

pub fn router() -> Router {
    Router::new().route("/login", post(login))
}

async fn login(cookies: Cookies, Json(payload): Json<LoginPayload>) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle post /login", "Handler");

    cookies.add(Cookie::new(
        payload.username.clone(),
        payload.password.clone(),
    ));

    Ok((StatusCode::OK, Json(payload)))
}
