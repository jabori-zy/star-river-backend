pub mod app_state;
pub mod market_engine;

use axum::{routing::get, Router};
use axum::extract::State;

use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use crate::app_state::AppState;
use tracing::Level;

#[tokio::main]
async fn main() {
    // 设置生产环境的日志级别
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        // build but do not install the subscriber.
        .init();

    // build our application with a route
    // 设置跨域
    let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods(Any)
    .allow_headers(Any);

    // 创建app状态
    let app_state = AppState::new();

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/get_app_state", get(get_app_state))
        .layer(cors)
        .with_state(app_state.clone());

    // 初始化app
    init_app(State(app_state)).await;

    // 允许从环境变量配置监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse::<SocketAddr>()
        .expect("Invalid server address");

    // run it
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn hello_world() -> String {
    "Hello, World!".to_string()
}

// 获取app状态
async fn get_app_state(State(app_state): State<AppState>) {
    tracing::info!("app_state: {:?}", app_state);
}

async fn init_app(State(mut app_state): State<AppState>) {
    app_state.heartbeat.start().await.unwrap();
    tracing::info!("heartbeat started");
}
