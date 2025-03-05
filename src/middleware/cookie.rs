use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
};
use sqlx::{Pool, Sqlite};

use crate::error::{Error, Result};

pub struct Cookies;

impl<S> FromRequestParts<S> for Cookies
where
    S: Send + Sync,
    Pool<Sqlite>: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        println!("--> {:<8} - Cookie", "Extractor");

        let cookies = parts
            .extract::<tower_cookies::Cookies>()
            .await
            .map_err(|_| Error::AuthErrorMissCookie)?;

        let State(pool) = parts
            .extract_with_state::<State<Pool<Sqlite>>, S>(state)
            .await
            .map_err(|_| Error::AuthErrorInvalidCookie)?;

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

        Ok(Cookies)
    }
}
