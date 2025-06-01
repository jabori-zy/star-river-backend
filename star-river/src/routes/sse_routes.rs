use axum::{
    routing::get,
    Router,
};
use crate::sse::{
    market_sse_handler, indicator_sse_handler, live_strategy_sse_handler
};
use crate::sse::account_sse::account_sse_handler;
use crate::sse::backtest_strategy_sse::backtest_strategy_sse_handler;
use crate::star_river::StarRiver;

pub fn create_sse_routes() -> Router<StarRiver> {
    Router::new()
        // 市场数据流
        .route("/market", get(market_sse_handler))
        
        // 指标数据流
        .route("/indicator", get(indicator_sse_handler))
        
        // 策略数据流
        .route("/strategy/live", get(live_strategy_sse_handler))
        .route("/strategy/backtest", get(backtest_strategy_sse_handler))
        
        // 账户数据流
        .route("/account", get(account_sse_handler))
}