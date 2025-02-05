use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
// use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(handler_get))
        .route("/users", post(handler_post))
        .route("/users", get(handler_get_query))
        .route("/users/{name}", get(handler_get_capture))
        .route_service("/src/main", get_service(ServeFile::new("./src/main.rs")))
        .fallback_service(get_service(ServeDir::new("./")))
        // .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handler_get() -> impl IntoResponse {
    (StatusCode::OK, Html("Hello world!"))
}

async fn handler_get_capture(Path(name): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, Html(format!("user {name}")))
}

#[derive(Serialize, Deserialize)]
struct QueryPayLoad {
    username: String,
}

async fn handler_get_query(Query(username): Query<QueryPayLoad>) -> impl IntoResponse {
    (StatusCode::FOUND, Json(username))
}

#[derive(Deserialize)]
struct UserPayLoad {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
    password: String,
}

async fn handler_post(Json(payload): Json<UserPayLoad>) -> impl IntoResponse {
    let user = User {
        id: 1337,
        username: payload.username,
        password: payload.password,
    };

    (StatusCode::CREATED, Json(user))
}

// async fn handler_post_cookies(
//     cookies: Cookies,
//     Json(payload): Json<UserPayLoad>,
// ) -> impl IntoResponse {
//     cookies.add(Cookie::new(payload.username.clone(), "New"));

//     let user = User {
//         id: 1337,
//         username: payload.username,
//         password: payload.password,
//     };

//     (StatusCode::CREATED, Json(user))
// }
