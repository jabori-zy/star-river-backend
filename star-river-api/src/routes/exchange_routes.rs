use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::exchange_api::{connect_exchange, get_exchange_status},
    star_river::StarRiver,
};

pub fn create_exchange_routes() -> Router<StarRiver> {
    Router::new()
        // 账户配置管理
        .route("/status/{account_id}", get(get_exchange_status))
        .route("/connect/{account_id}", post(connect_exchange))
}
