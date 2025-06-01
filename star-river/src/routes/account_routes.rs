use axum::{
    routing::{get, post, delete, put},
    Router,
};

use crate::api::account_api::{start_mt5_terminal, add_account_config, delete_account_config, update_account_config, update_account_is_available, get_account_configs};
use crate::star_river::StarRiver;

pub fn create_account_routes() -> Router<StarRiver> {
    Router::new()
        // 账户配置管理
        .route("/config", get(get_account_configs))
        .route("/config", post(add_account_config))
        .route("/config/{account_id}", delete(delete_account_config))
        .route("/config/{account_id}", put(update_account_config))
        .route("/config/{account_id}/availability", post(update_account_is_available))
        
        // MT5账户操作
        .route("/start_mt5_terminal", post(start_mt5_terminal))
}