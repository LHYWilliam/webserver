use axum::{
    Router,
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use http::{
    model::ticket::TicketController,
    web::{login, ticket},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = TicketController::new().await.unwrap();

    let app = Router::new()
        .route("/", get(handler_root))
        .merge(login::router())
        .merge(ticket::router(controller))
        .layer(CookieManagerLayer::new())
        .layer(middleware::map_request(requset_input))
        .layer(middleware::map_response(response_output));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("--> {:<8} - {}\n", "Listener", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler_root() -> impl IntoResponse {
    println!("--> {:<8} - handle get /", "Handler");

    (StatusCode::OK, Html("Hello, World!"))
}

async fn requset_input(requset: Request<Body>) -> Request<Body> {
    println!("--> {:<8} - requset input", "Mapper");

    requset
}

async fn response_output(response: Response<Body>) -> Response<Body> {
    println!("--> {:<8} - response output\n", "Mapper");

    response
}
