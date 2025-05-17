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


#[derive(Debug, Deserialize)]
pub struct GetAllCacheKeyParams {
    pub key_type: Option<String>,
}

// 初始化策略
pub async fn get_cache_key(State(star_river): State<StarRiver>, Query(params): Query<GetAllCacheKeyParams>) -> (StatusCode, Json<ApiResponse<Vec<String>>>) {
    let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
    let engine_guard = engine.lock().await;
    let cache_key = match params.key_type {
        Some(key_type) => {
            match key_type.as_str() {
                "kline" => engine_guard.get_cache_key(Some(key_type.as_str())).await.unwrap(),
                "indicator" => engine_guard.get_cache_key(Some(key_type.as_str())).await.unwrap(),
                _ => return (StatusCode::BAD_REQUEST, Json(ApiResponse {
                    code: 1,
                    message: "invalid key type".to_string(),
                    data: None,
                })),
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

#[derive(Deserialize, Debug)]
pub struct GetStrategyCacheParams {
    pub cache_key: String,
}

pub async fn get_cache_value(
    State(star_river): State<StarRiver>, 
    Query(params): Query<GetStrategyCacheParams>
) -> (StatusCode, Json<ApiResponse<Vec<Vec<f64>>>>) {
    let cache_key = params.cache_key;
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::CacheEngine).await;
    let mut engine_guard = engine.lock().await;
    let cache_engine = engine_guard.as_any_mut().downcast_mut::<CacheEngine>().unwrap();
    let cache_key = CacheKey::from_str(&cache_key).unwrap();
    let cache = cache_engine.get_cache_value(&cache_key, None).await;
    let cache_values: Vec<Vec<f64>> = cache.iter().map(|cache_value| cache_value.to_list()).collect();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: Some(cache_values),
    }))
}


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




