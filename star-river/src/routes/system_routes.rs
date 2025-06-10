use axum::{
    routing::{put, get},
    Router,
};
use crate::star_river::StarRiver;
use crate::api::system_api::{update_system_config, get_system_config};

pub fn create_system_routes() -> Router<StarRiver> {
    Router::new()
        // 系统配置管理
        .route("/config", put(update_system_config))
        .route("/config", get(get_system_config))
}