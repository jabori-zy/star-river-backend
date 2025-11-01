use crate::api::system_api::{get_system_config, get_timezones, update_system_config};
use crate::star_river::StarRiver;
use axum::{
    Router,
    routing::{get, put},
};

pub fn create_system_routes() -> Router<StarRiver> {
    Router::new()
        // 系统配置管理
        .route("/config", put(update_system_config))
        .route("/config", get(get_system_config))
        .route("/timezones", get(get_timezones))
}
