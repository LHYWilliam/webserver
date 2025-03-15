use std::ops::Deref;

use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
};
use sqlx::{Pool, Sqlite};
use tracing::info;

use crate::error::{AuthError, Error, Result};

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
        info!("[{:^12}] ┃ Cookie", "Middleware");

        let cookies = tower_cookies::Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidCookie)?;

        let State(pool) = State::<Pool<Sqlite>>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Unknown)?;

        let username = cookies
            .get("user")
            .map(|c| c.value().to_string())
            .ok_or(AuthError::InvalidCookie)?;

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
        .map_err(|_| AuthError::InvalidCookie)?;

        Ok(Cookies(cookies))
    }
}
