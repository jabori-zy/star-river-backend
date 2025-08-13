use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::api::strategy_api::{
    run_strategy, stop_strategy, init_strategy, get_strategy_cache_keys,
    enable_strategy_data_push, disable_strategy_data_push,
    create_strategy, update_strategy, delete_strategy, get_strategy_list, get_strategy_by_id
};

use crate::api::strategy_api::backtest_strategy::{update_backtest_chart_config, get_backtest_chart_config, play, pause, play_one, reset, get_play_index, get_virtual_orders, get_current_positions};

use crate::star_river::StarRiver;

pub fn create_strategy_routes() -> Router<StarRiver> {
    Router::new()
        // 策略管理
        .route("/", post(create_strategy))
        .route("/", get(get_strategy_list))
        .route("/{strategy_id}", get(get_strategy_by_id))
        .route("/{strategy_id}", post(update_strategy))
        .route("/{strategy_id}", delete(delete_strategy))
        
        // 策略生命周期管理
        .route("/{strategy_id}/init", post(init_strategy))
        .route("/{strategy_id}/stop", post(stop_strategy))
        // 策略缓存
        .route("/{strategy_id}/cache-keys", get(get_strategy_cache_keys))
}


pub fn create_live_strategy_routes() -> Router<StarRiver> {
    Router::new()
        .route("/{strategy_id}/run", post(run_strategy))
        // 策略数据推送控制
        .route("/{strategy_id}/data-push/enable", post(enable_strategy_data_push))
        .route("/{strategy_id}/data-push/disable", post(disable_strategy_data_push))
}

pub fn create_backtest_strategy_routes() -> Router<StarRiver> {
    Router::new()
        // 策略控制 (实时/回测)
        .route("/{strategy_id}/play", post(play))
        .route("/{strategy_id}/pause", post(pause))
        .route("/{strategy_id}/play-one", post(play_one))
        .route("/{strategy_id}/reset", post(reset))
        .route("/{strategy_id}/chart_config", post(update_backtest_chart_config))
        .route("/{strategy_id}/chart_config", get(get_backtest_chart_config))
        .route("/{strategy_id}/play-index", get(get_play_index))
        .route("/{strategy_id}/virtual-orders", get(get_virtual_orders))
        .route("/{strategy_id}/current-positions", get(get_current_positions))
}
