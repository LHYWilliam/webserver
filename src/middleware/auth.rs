use axum::{extract::Request, middleware::Next, response::Response};
use tower_cookies::Cookies;

use crate::error::{Error, Result};

pub async fn auth(cookies: Cookies, requset: Request, next: Next) -> Result<Response> {
    println!("--> {:<8} - auth", "Middleware");

    cookies.get("william").ok_or(Error::InvalidAuth)?;

    Ok(next.run(requset).await)
}
