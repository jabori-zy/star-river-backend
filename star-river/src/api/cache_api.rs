use crate::star_river::StarRiver;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::extract::State;
use axum::extract::Json;
use crate::api::response::ApiResponse;
use serde::Deserialize;
use std::collections::HashMap;
use engine::cache_engine::CacheEngine;
use types::cache::CacheKey;
use std::str::FromStr;
use types::engine::EngineName;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
#[schema(
    title = "缓存键类型",
    description = "缓存键类型"
)]
pub enum CacheKeyType {
    /// k线类型
    #[serde(rename = "kline")]
    Kline,
    /// 指标类型
    #[serde(rename = "indicator")]
    Indicator,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "获取所有缓存键参数",
    description = "获取所有缓存键"
)]
pub struct GetAllCacheKeyParams {
    #[schema(example = "kline")]
    pub key_type: Option<CacheKeyType>,
}




/// 获取所有缓存键
#[utoipa::path(
    get,
    path = "/api/v1/cache/keys",
    params(GetAllCacheKeyParams),
    tag = "缓存引擎",
    summary = "获取所有缓存键",
    responses(
        (status = 200, body = ApiResponse<Vec<String>>)
    )
)]
pub async fn get_cache_keys(State(star_river): State<StarRiver>, Query(params): Query<GetAllCacheKeyParams>) -> (StatusCode, Json<ApiResponse<Vec<String>>>) {
    let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
    let engine_guard = engine.lock().await;
    let cache_key = match params.key_type {
        Some(key_type) => {
            match key_type {
                CacheKeyType::Kline => engine_guard.get_cache_key(Some("kline")).await.unwrap(),
                CacheKeyType::Indicator => engine_guard.get_cache_key(Some("indicator")).await.unwrap(),
            }
        }
        None => engine_guard.get_cache_key(None).await.unwrap(),
    };
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: Some(cache_key),
    }))
}

#[derive(Deserialize, Debug, IntoParams, ToSchema)]
#[schema(
    title = "获取缓存值参数",
    description = "获取缓存值"
)]
pub struct GetCacheValueParams {
    pub cache_key: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/cache/value",
    params(GetCacheValueParams),
    tag = "缓存引擎",
    summary = "获取缓存值",
    responses(
        (status = 200, body = ApiResponse<Vec<Vec<f64>>>)
    )
)]
pub async fn get_cache_value(
    State(star_river): State<StarRiver>, 
    Query(params): Query<GetCacheValueParams>
) -> (StatusCode, Json<ApiResponse<Vec<Vec<f64>>>>) {
    let cache_key = params.cache_key;
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::CacheEngine).await;
    let mut engine_guard = engine.lock().await;
    let cache_engine = engine_guard.as_any_mut().downcast_mut::<CacheEngine>().unwrap();
    let cache_key = CacheKey::from_str(&cache_key).unwrap();
    let cache = cache_engine.get_cache_value(&cache_key, None, None).await;
    let cache_values: Vec<Vec<f64>> = cache.iter().map(|cache_value| cache_value.to_list()).collect();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: Some(cache_values),
    }))
}



#[utoipa::path(
    get,
    path = "/api/v1/cache/memory_size",
    tag = "缓存引擎",
    summary = "获取缓存内存大小",
    responses(
        (status = 200, body = ApiResponse<HashMap<String, u32>>)
    )
)]
pub async fn get_memory_size(State(star_river): State<StarRiver>) -> (StatusCode, Json<ApiResponse<HashMap<String, u32>>>) {
    let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
    let engine_guard = engine.lock().await;
    let memory_size = engine_guard.get_memory_size().await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: Some(memory_size),
    }))
}




