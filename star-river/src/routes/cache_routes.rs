use axum::{
    routing::get,
    Router,
};
use crate::api::cache_api::{get_cache_keys, get_memory_size, get_cache_value};
use crate::star_river::StarRiver;

pub fn create_cache_routes() -> Router<StarRiver> {
    Router::new()
        .route("/keys", get(get_cache_keys))
        .route("/memory-size", get(get_memory_size))
        .route("/value", get(get_cache_value))
}