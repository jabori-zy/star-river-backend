
pub mod star_river;
pub mod market_engine;
pub mod api;
pub mod websocket;
pub mod sse;


use axum::{routing::{get, post}, Router, routing::any};
use axum::extract::State;

use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use crate::star_river::StarRiver;
use crate::api::market_api::subscribe_kline_stream;
use crate::api::indicator_api::subscribe_indicator;
use crate::api::market_api::get_heartbeat_lock;
use crate::sse::{market_sse_handler, indicator_sse_handler};
use tracing::Level;
use crate::websocket::ws_handler;
use crate::star_river::init_app;

#[tokio::main]
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
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
    let star_river = StarRiver::new();

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/subscribe_kline_stream", get(subscribe_kline_stream))
        .route("/get_heartbeat_lock", get(get_heartbeat_lock))
        .route("/subscribe_indicator", post(subscribe_indicator))
        .route("/ws", any(ws_handler))
        .route("/market_sse", get(market_sse_handler))
        .route("/indicator_sse", get(indicator_sse_handler))
        .layer(cors)
        .with_state(star_river.clone());

    // 初始化app
    init_app(State(star_river)).await;

    


    // 允许从环境变量配置监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse::<SocketAddr>()
        .expect("Invalid server address");

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    Ok(())

}

async fn hello_world() -> String {
    tracing::info!("hello_world");
    "Hello, World!".to_string()
}







