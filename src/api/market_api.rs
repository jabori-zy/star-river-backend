use std::str::FromStr;

use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use types::market::{Exchange, KlineInterval};
use axum::response::IntoResponse;
use serde::Deserialize;
use axum::extract::Query;
use event_center::Channel;
use types::indicator::Indicators;

#[derive(Deserialize)]
pub struct KlineParams {
    exchange: String,
    symbol: String,
    interval: String,
}

#[axum::debug_handler]
pub async fn subscribe_kline_stream(State(star_river): State<StarRiver>, Query(params): Query<KlineParams>) -> impl IntoResponse {
    tracing::info!("subscribe_kline_stream");
    let exchange = Exchange::from_str(&params.exchange).expect("Invalid exchange");
    let interval = KlineInterval::from_str(&params.interval).expect("Invalid kline interval");

    let binance_publisher = star_river.event_center.lock().await.get_publisher(Channel::Market).unwrap();
    // 注册交易所
    let market_engine = star_river.market_engine.clone();
    let heartbeat = {
        let heartbeat = star_river.heartbeat.lock().await;
        heartbeat
    };
    heartbeat.run_async_task_once("订阅k线".to_string(), async move {
        let mut market_engine = market_engine.lock().await;
        market_engine.register_exchange(exchange.clone(), binance_publisher).await.expect("Failed to register exchange");

        if let Err(e) = market_engine.get_kline_series(exchange.clone(), params.symbol.clone(), interval.clone(), None, None, Some(1)).await {
            tracing::error!("Failed to get kline series: {}", e);
        };

        if let Err(e) = market_engine.subscribe_kline_stream(exchange.clone(), params.symbol.clone(), interval.clone()).await {
            tracing::error!("Failed to subscribe kline stream: {}", e);
        };
        
        if let Err(e) = market_engine.get_socket_stream(exchange.clone()).await {
            tracing::error!("Failed to get socket stream: {}", e);
        };
    }).await;
    StatusCode::OK
}


pub async fn get_heartbeat_lock(State(star_river): State<StarRiver>) -> impl IntoResponse {
    let heartbeat = star_river.heartbeat.try_lock();
    if let Ok(_) = heartbeat {
        tracing::info!("获取心跳锁成功");
    } else {
        tracing::error!("获取心跳锁失败");
    }
    StatusCode::OK
}


#[derive(Deserialize, Debug)]
pub struct IndicatorParams {
    exchange: String,
    symbol: String,
    interval: String,
    indicator: Indicators,
}

#[axum::debug_handler]
pub async fn subscribe_indicator(State(star_river): State<StarRiver>, Query(params): Query<IndicatorParams>) -> impl IntoResponse {
    tracing::info!("订阅指标");
    tracing::info!("params: {:?}", params);
    let exchange = Exchange::from_str(&params.exchange).expect("Invalid exchange");
    let symbol = params.symbol.clone();
    let indicator = params.indicator.clone();
    let interval = KlineInterval::from_str(&params.interval).expect("Invalid kline interval");
    let market_engine = star_river.market_engine.clone();
    let heartbeat = {
        let heartbeat = star_river.heartbeat.lock().await;
        heartbeat
    };
    heartbeat.run_async_task_once("订阅指标".to_string(), async move {
        let mut market_engine = market_engine.lock().await;
        market_engine.subscribe_indicator(exchange.clone(), symbol.clone(), interval.clone(), indicator.clone()).await.expect("Failed to subscribe indicator");
    }).await;
    StatusCode::OK
}

