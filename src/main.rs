use std::{env, net::SocketAddr, result};

use axum::{
    Router,
    body::{self, Body},
    extract::Request,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;

use webserver::{
    error::{Error, Result},
    web::{login, register, room, ticket},
};

#[tokio::main]
async fn main() -> result::Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_level(true)
        .init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .route("/", routing::get(handler_root))
        .merge(register::router(pool.clone()))
        .merge(login::router(pool.clone()))
        .merge(ticket::router(pool.clone()))
        .merge(room::router())
        .layer(CookieManagerLayer::new())
        .layer(middleware::map_request(requset_input))
        .layer(middleware::map_response(response_output))
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    info!("[{:^12}] - {}", "Listener", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler_root() -> impl IntoResponse {
    info!("[{:^12}] - handle get /", "Handler");

    (StatusCode::OK, "Hello, World!")
}

async fn requset_input(requset: Request<Body>) -> Request<Body> {
    let method = requset.method();
    let uri = requset.uri();

    info!(
        "[{:^12}] - ====== method: {:?}, uri: {:?}",
        "Input", method, uri
    );

    requset
}

async fn response_output(response: Response) -> Result<impl IntoResponse> {
    let version = response.version();
    let status = response.status();
    let headers = response.headers().clone();

    let Ok(bytes) = body::to_bytes(response.into_body(), usize::MAX).await else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .map_err(|_| Error::Unknown);
    };

    let body = String::from_utf8_lossy(&bytes);
    info!(
        "[{:^12}] - ====== status: {}, body: {:?}",
        "Output", status, body
    );

    let mut builder = Response::builder().version(version).status(status);

    for (name, value) in headers.iter() {
        builder = builder.header(name, value);
    }

    builder.body(Body::from(bytes)).map_err(|_| Error::Unknown)
}
