use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    api::strategy_api::{
        backtest::*,
        strategy_management::{create_strategy, delete_strategy, get_strategy_by_id, get_strategy_list, update_strategy},
    },
    star_river::StarRiver,
};

pub fn create_strategy_routes() -> Router<StarRiver> {
    Router::new()
        // Strategy management
        .route("/", post(create_strategy))
        .route("/", get(get_strategy_list))
        .route("/{strategy_id}", get(get_strategy_by_id))
        .route("/{strategy_id}", post(update_strategy))
        .route("/{strategy_id}", delete(delete_strategy))
        // Strategy lifecycle management
        // Strategy cache
        .route("/{strategy_id}/cache-keys", get(get_strategy_keys))
}

pub fn create_backtest_strategy_routes() -> Router<StarRiver> {
    Router::new()
        .route("/{strategy_id}/init", post(init_strategy))
        .route("/{strategy_id}/stop", post(stop_strategy))
        .route("/{strategy_id}/play", post(play))
        .route("/{strategy_id}/pause", post(pause))
        .route("/{strategy_id}/play-one", post(play_one))
        .route("/{strategy_id}/reset", post(reset))
        .route("/{strategy_id}/chart_config", post(update_backtest_chart_config))
        .route("/{strategy_id}/chart_config", get(get_backtest_chart_config))
        .route("/{strategy_id}/play-index", get(get_cycle_id))
        .route("/{strategy_id}/strategy-datetime", get(get_strategy_datetime))
        .route("/{strategy_id}/virtual-orders", get(get_virtual_orders))
        .route("/{strategy_id}/current-positions", get(get_current_positions))
        .route("/{strategy_id}/history-positions", get(get_history_positions))
        .route("/{strategy_id}/stats-history", get(get_stats_history))
        .route("/{strategy_id}/virtual-transactions", get(get_virtual_transactions))
        .route("/{strategy_id}/run-state", get(get_strategy_run_state))
        .route("/{strategy_id}/running-log", get(get_running_log))
        .route("/{strategy_id}/data", get(get_strategy_data))
        .route("/{strategy_id}/variable", get(get_strategy_variable))
        .route("/{strategy_id}/performance-report", get(get_strategy_performance_report))
}
