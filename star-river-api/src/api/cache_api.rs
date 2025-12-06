use crate::api::response::ApiResponse;
use crate::star_river::StarRiver;
use axum::extract::Json;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use engine::cache_engine::CacheEngine;
use serde::Deserialize;
use star_river_core::engine::EngineName;
use star_river_core::key::Key;
use std::collections::HashMap;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
#[schema(title = "Cache key type", description = "Cache key type")]
pub enum CacheKeyType {
    /// Kline type
    #[serde(rename = "kline")]
    Kline,
    /// Indicator type
    #[serde(rename = "indicator")]
    Indicator,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[schema(title = "Get all cache keys parameters", description = "Get all cache keys")]
pub struct GetAllCacheKeyParams {
    #[schema(example = "kline")]
    pub key_type: Option<CacheKeyType>,
}

/// Get all cache keys
#[utoipa::path(
    get,
    path = "/api/v1/cache/keys",
    params(GetAllCacheKeyParams),
    tag = "Cache Engine",
    summary = "Get all cache keys",
    responses(
        (status = 200, body = ApiResponse<Vec<String>>)
    )
)]
pub async fn get_cache_keys(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetAllCacheKeyParams>,
) -> (StatusCode, Json<ApiResponse<Vec<String>>>) {
    let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
    let engine_guard = engine.lock().await;
    let cache_key = match params.key_type {
        Some(key_type) => match key_type {
            CacheKeyType::Kline => engine_guard.get_key(Some("kline")).await.unwrap(),
            CacheKeyType::Indicator => engine_guard.get_key(Some("indicator")).await.unwrap(),
        },
        None => engine_guard.get_key(None).await.unwrap(),
    };
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(cache_key),
        }),
    )
}

#[derive(Deserialize, Debug, IntoParams, ToSchema)]
#[schema(title = "Get cache value parameters", description = "Get cache value")]
pub struct GetCacheValueParams {
    pub key: String,
    pub index: Option<u32>, // Start index
    pub limit: Option<u32>, // Length
}

#[utoipa::path(
    get,
    path = "/api/v1/cache/value",
    params(GetCacheValueParams),
    tag = "Cache Engine",
    summary = "Get cache value",
    responses(
        (status = 200, body = ApiResponse<Vec<utoipa::openapi::Object>>)
    )
)]
pub async fn get_cache_value(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetCacheValueParams>,
) -> (StatusCode, Json<ApiResponse<Vec<serde_json::Value>>>) {
    let key_str = params.key;
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::CacheEngine).await;
    let mut engine_guard = engine.lock().await;
    let cache_engine = engine_guard.as_any_mut().downcast_mut::<CacheEngine>().unwrap();
    let key = Key::from_str(&key_str).unwrap();
    let value = cache_engine.get_cache_value(&key, params.index, params.limit).await.unwrap();
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(value),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/cache/memory_size",
    tag = "Cache Engine",
    summary = "Get cache memory size",
    responses(
        (status = 200, body = ApiResponse<HashMap<String, u32>>)
    )
)]
pub async fn get_memory_size(State(star_river): State<StarRiver>) -> (StatusCode, Json<ApiResponse<HashMap<String, u32>>>) {
    let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
    let engine_guard = engine.lock().await;
    let memory_size = engine_guard.get_memory_size().await.unwrap();
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(memory_size),
        }),
    )
}
