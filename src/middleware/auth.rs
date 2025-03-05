use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use sqlx::{Pool, Sqlite};
use tower_cookies::Cookies;

use crate::{
    error::{Error, Result},
    middleware::jwt::Claims,
};

pub async fn auth(
    _claims: Claims,
    cookies: Cookies,
    State(pool): State<Pool<Sqlite>>,
    requset: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - auth", "Middleware");

    let username = cookies
        .get("user")
        .ok_or(Error::AuthErrorMissCookie)?
        .value()
        .to_string();

    sqlx::query!(
        r#"
        select *
        from users
        where username = ?
        "#,
        username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::AuthErrorInvalidCookie)?;

    Ok(next.run(requset).await)
}
