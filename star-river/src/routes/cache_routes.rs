use crate::api::cache_api::{get_cache_keys, get_cache_value, get_memory_size};
use crate::star_river::StarRiver;
use axum::{Router, routing::get};

pub fn create_cache_routes() -> Router<StarRiver> {
    Router::new()
        .route("/keys", get(get_cache_keys))
        .route("/memory_size", get(get_memory_size))
        .route("/value", get(get_cache_value))
}
