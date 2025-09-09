use crate::api::system_api::{get_system_config, update_system_config};
use crate::star_river::StarRiver;
use axum::{
    routing::{get, put},
    Router,
};

pub fn create_system_routes() -> Router<StarRiver> {
    Router::new()
        // 系统配置管理
        .route("/config", put(update_system_config))
        .route("/config", get(get_system_config))
}
