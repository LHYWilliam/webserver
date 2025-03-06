use std::ops::Deref;

use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
};
use sqlx::{Pool, Sqlite};

use crate::error::{Error, Result};

pub struct Cookies(tower_cookies::Cookies);

impl Deref for Cookies {
    type Target = tower_cookies::Cookies;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
            .map_err(|_| Error::AuthErrorInvalidCookie)?;

        let State(pool) = parts
            .extract_with_state::<State<Pool<Sqlite>>, S>(state)
            .await
            .map_err(|_| Error::ExtractorError)?;

        let username = cookies
            .get("user")
            .map(|c| c.value().to_string())
            .ok_or(Error::AuthErrorInvalidCookie)?;

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

        Ok(Cookies(cookies))
    }
}
