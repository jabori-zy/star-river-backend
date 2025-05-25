use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::api::mutation_api::account_mutation::{
    add_account_config, delete_account_config, update_account_config,
    update_account_config_is_available
};
use crate::api::query_api::get_account_config;
use crate::api::account_api::login_mt5_account;
use crate::star_river::StarRiver;

pub fn create_account_routes() -> Router<StarRiver> {
    Router::new()
        // 账户配置管理
        .route("/config", post(add_account_config))
        .route("/config", get(get_account_config))
        .route("/config/{id}", delete(delete_account_config))
        .route("/config/{id}", post(update_account_config))
        .route("/config/{id}/availability", post(update_account_config_is_available))
        
        // MT5账户操作
        .route("/mt5/login", post(login_mt5_account))
}