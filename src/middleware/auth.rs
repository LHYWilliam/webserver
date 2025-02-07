use axum::{extract::Request, middleware::Next, response::Response};
use tower_cookies::Cookies;

use crate::error::{Error, Result};

pub async fn auth(cookies: Cookies, requset: Request, next: Next) -> Result<Response> {
    print!("--> {:<8} - auth\n", "Middleware");

    cookies.get("william").ok_or(Error::InvalidAuth)?;

    Ok(next.run(requset).await)
}
