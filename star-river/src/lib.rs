
pub mod star_river;
pub mod api;
pub mod websocket;
pub mod sse;


use axum::{routing::{get, post, delete}, Router, routing::any};
use axum::extract::State;

use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use crate::star_river::StarRiver;
use crate::api::mutation_api::{create_strategy, update_strategy, delete_strategy};
use crate::api::query_api::{get_strategy_list, get_strategy_by_id};
use crate::api::strategy_api::{run_strategy, stop_strategy, init_strategy, enable_strategy_event_push, disable_strategy_event_push};
use crate::sse::{market_sse_handler, indicator_sse_handler, strategy_sse_handler};
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
    let star_river = StarRiver::new().await;

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/ws", any(ws_handler))
        .route("/market_sse", get(market_sse_handler))
        .route("/indicator_sse", get(indicator_sse_handler))
        .route("/strategy_sse", get(strategy_sse_handler))
        .route("/create_strategy", post(create_strategy))
        .route("/init_strategy", post(init_strategy))
        .route("/run_strategy", post(run_strategy))
        .route("/stop_strategy", post(stop_strategy))
        .route("/enable_strategy_event_push", post(enable_strategy_event_push))
        .route("/disable_strategy_event_push", post(disable_strategy_event_push))
        .route("/get_strategy_list", get(get_strategy_list))
        .route("/update_strategy", post(update_strategy))
        .route("/delete_strategy", delete(delete_strategy))
        .route("/get_strategy", get(get_strategy_by_id))
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







