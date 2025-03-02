use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

use crate::error::Result;

#[derive(Serialize, Deserialize)]
struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub fn router() -> Router {
    Router::new().route("/login", routing::post(login))
}

async fn login(cookies: Cookies, Json(payload): Json<LoginPayload>) -> Result<impl IntoResponse> {
    println!("--> {:<8} - handle post /login", "Handler");

    cookies.add(Cookie::new(
        payload.username.clone(),
        payload.password.clone(),
    ));

    Ok((StatusCode::OK, Json(payload)))
}
