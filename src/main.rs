pub mod star_river;
pub mod market_engine;
pub mod api;

use axum::{routing::get, Router};
use axum::extract::State;

use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use crate::star_river::StarRiver;
use crate::api::market_api::subscribe_kline_stream;
use crate::api::market_api::subscribe_indicator;
use crate::api::market_api::get_heartbeat_lock;
use tracing::Level;
use event_center::Channel;

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
    let star_river = StarRiver::new();

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/subscribe_kline_stream", get(subscribe_kline_stream))
        .route("/get_heartbeat_lock", get(get_heartbeat_lock))
        .route("/subscribe_indicator", get(subscribe_indicator))
        .layer(cors)
        .with_state(star_river.clone());

    // 初始化app
    start_heartbeat(State(star_river.clone())).await;
    init_app(State(star_river)).await;

    


    // 允许从环境变量配置监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse::<SocketAddr>()
        .expect("Invalid server address");

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .unwrap();

    
    
}

async fn hello_world() -> String {
    tracing::info!("hello_world");
    "Hello, World!".to_string()
}



async fn init_app(State(app_state): State<StarRiver>) {
    tokio::spawn(async move {
        let market_event_receiver = app_state.event_center.lock().await.subscribe(Channel::Market).unwrap();
        let command_event_receiver = app_state.event_center.lock().await.subscribe(Channel::Command).unwrap();
        // 启动缓存引擎
        let mut cache_engine = app_state.cache_engine.lock().await;
        cache_engine.start(market_event_receiver, command_event_receiver).await;

        // 启动指标引擎
        let indicator_event_receiver = app_state.event_center.lock().await.subscribe(Channel::Indicator).unwrap();
        let indicator_engine = app_state.indicator_engine.lock().await;
        indicator_engine.listen(indicator_event_receiver).await;
    });
}


async fn start_heartbeat(star_river: State<StarRiver>) {
    let heartbeat = star_river.heartbeat.clone();
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
        tracing::info!("心跳已启动");
    });
}
