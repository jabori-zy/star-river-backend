use types::indicator::Indicators;
use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use types::market::{Exchange, KlineInterval};
use axum::response::IntoResponse;
use serde::Deserialize;
use axum::extract::Json;
use std::str::FromStr;


#[derive(Deserialize, Debug)]
pub struct IndicatorParams {
    exchange: String,
    symbol: String,
    interval: String,
    indicator: Indicators,
}

#[axum::debug_handler]
pub async fn subscribe_indicator(State(star_river): State<StarRiver>, Json(params): Json<IndicatorParams>) -> impl IntoResponse {
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