use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::api::mutation_api::{create_strategy, update_strategy, delete_strategy};
use crate::api::query_api::{get_strategy_list, get_strategy_by_id};
use crate::api::strategy_api::{
    run_strategy, stop_strategy, init_strategy, get_strategy_cache_keys,
    enable_strategy_data_push, disable_strategy_data_push,
    play, pause, play_one, stop
};
use crate::star_river::StarRiver;

pub fn create_strategy_routes() -> Router<StarRiver> {
    Router::new()
        // 策略管理
        .route("/", post(create_strategy))
        .route("/", get(get_strategy_list))
        .route("/{id}", get(get_strategy_by_id))
        .route("/{id}", post(update_strategy))
        .route("/{id}", delete(delete_strategy))
        
        // 策略生命周期管理
        .route("/{id}/init", post(init_strategy))
        // 策略缓存
        .route("/{id}/cache-keys", get(get_strategy_cache_keys))
}


pub fn create_live_strategy_routes() -> Router<StarRiver> {
    Router::new()
        .route("/{id}/run", post(run_strategy))
        .route("/{id}/stop", post(stop_strategy))
        // 策略数据推送控制
        .route("/{id}/data-push/enable", post(enable_strategy_data_push))
        .route("/{id}/data-push/disable", post(disable_strategy_data_push))
}

pub fn create_backtest_strategy_routes() -> Router<StarRiver> {
    Router::new()
        // 策略控制 (实时/回测)
        .route("/{id}/play", post(play))
        .route("/{id}/pause", post(pause))
        .route("/{id}/play-one", post(play_one))
        .route("/{id}/stop-playback", post(stop))
                
}
