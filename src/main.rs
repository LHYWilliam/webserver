use std::env;

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use http::web::{login, register, ticket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .route("/", routing::get(handler_root))
        .merge(register::router(pool.clone()))
        .merge(login::router(pool.clone()))
        .merge(ticket::router(pool.clone()))
        .layer(CookieManagerLayer::new())
        .layer(middleware::map_request(requset_input))
        .layer(middleware::map_response(response_output));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("[{:^12}] - {}\n", "Listener", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler_root() -> impl IntoResponse {
    println!("[{:^12}] - handle get /", "Handler");

    (StatusCode::OK, "Hello, World!")
}

async fn requset_input(requset: Request<Body>) -> Request<Body> {
    println!("[{:^12}] - requset input", "Mapper");

    requset
}

async fn response_output(response: Response) -> impl IntoResponse {
    println!("[{:^12}] - response output\n", "Mapper");

    response
}
