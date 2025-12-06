use axum::{Router, routing::get};

use crate::sse::{
    account_sse_handler,
    backtest_strategy_event_sse_handler,
    backtest_strategy_performance_sse_handler,
    backtest_strategy_running_log_sse_handler,
    backtest_strategy_state_log_sse_handler,
    indicator_sse_handler,
    // live_strategy_sse_handler,
    market_sse_handler,
};
use crate::star_river::StarRiver;

pub fn create_sse_routes() -> Router<StarRiver> {
    Router::new()
        // Market data stream
        .route("/market", get(market_sse_handler))
        // Indicator data stream
        .route("/indicator", get(indicator_sse_handler))
        // Strategy data stream
        // .route("/strategy/live", get(live_strategy_sse_handler))
        .route("/strategy/backtest/event", get(backtest_strategy_event_sse_handler))
        .route("/strategy/backtest/state-log", get(backtest_strategy_state_log_sse_handler))
        .route("/strategy/backtest/running-log", get(backtest_strategy_running_log_sse_handler))
        .route("/strategy/backtest/performance", get(backtest_strategy_performance_sse_handler))
        // Account data stream
        .route("/account", get(account_sse_handler))
}
