pub mod cookie;
pub mod jwt;

use axum::{extract::Request, middleware::Next, response::IntoResponse};

use crate::{
    error::Result,
    middleware::auth::{cookie::Cookies, jwt::Claims},
};

pub async fn auth(
    _claims: Claims,
    _cookies: Cookies,
    requset: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    println!("[{:^12}] - auth", "Middleware");

    Ok(next.run(requset).await)
}
