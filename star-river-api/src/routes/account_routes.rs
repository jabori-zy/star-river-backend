use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    api::account_api::{
        add_account_config, delete_account_config, get_account_config_list, update_account_config, update_account_is_available,
    },
    star_river::StarRiver,
};

pub fn create_account_routes() -> Router<StarRiver> {
    Router::new()
        // 账户配置管理
        .route("/config", get(get_account_config_list))
        .route("/config", post(add_account_config))
        .route("/config/{account_id}", delete(delete_account_config))
        .route("/config/{account_id}", put(update_account_config))
        .route("/config/{account_id}/availability", post(update_account_is_available))
    // MT5账户操作
    // .route("/start_mt5_terminal", post(start_mt5_terminal))
}
