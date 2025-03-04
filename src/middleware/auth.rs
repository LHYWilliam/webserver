use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use sqlx::{FromRow, Pool, Sqlite};
use tower_cookies::Cookies;

use crate::error::{Error, Result};

#[derive(FromRow)]
struct Password {
    pub password: String,
}

pub async fn auth(
    cookies: Cookies,
    State(pool): State<Pool<Sqlite>>,
    requset: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    println!("--> {:<8} - auth", "Middleware");

    let username = cookies
        .get("username")
        .ok_or(Error::InvalidAuth)?
        .value()
        .to_string();
    let password = cookies
        .get("password")
        .ok_or(Error::InvalidAuth)?
        .value()
        .to_string();

    let password_query = sqlx::query_as!(
        Password,
        r#"
        select password
        from users
        where username = ?
        "#,
        username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::WorngUsernameOrPassword)?;

    match password_query.password == password {
        true => Ok(next.run(requset).await),
        false => Err(Error::WorngUsernameOrPassword),
    }
}
