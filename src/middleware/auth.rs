use axum::{extract::Request, middleware::Next, response::IntoResponse};

use crate::{
    error::Result,
    middleware::{cookie::Cookies, jwt::Claims},
};

pub async fn auth(
    _claims: Claims,
    _cookies: Cookies,
    requset: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - auth", "Middleware");

    Ok(next.run(requset).await)
}
