use crate::star_river::StarRiver;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::extract::State;
use axum::extract::Json;
use crate::api::response::ApiResponse;
use serde::Deserialize;
use types::cache::CacheKey;
use types::cache::cache_key::{KlineCacheKey, IndicatorCacheKey};
use types::cache::CacheValue;
use std::collections::HashMap;

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

// #[derive(Debug, Deserialize)]
// pub struct GetCacheValueParams {
//     pub exchange: String,
//     pub symbol: String,
//     pub interval: String,
//     pub indicator: Option<String>,
//     pub limit: Option<u32>,
// }

// pub async fn get_cache_value(State(star_river): State<StarRiver>, Query(params): Query<GetCacheValueParams>) -> (StatusCode, Json<ApiResponse<Vec<CacheValue>>>) {
//     let engine = star_river.engine_manager.lock().await.get_cache_engine().await;
//     let engine_guard = engine.lock().await;
//     if params.indicator.is_some() {
//         let cache_key = CacheKey::Indicator(IndicatorCacheKey::new(params.exchange, params.symbol, params.interval, params.indicator.unwrap()));
//         let cache_value = engine_guard.get_cache_value(&cache_key, params.limit).await.unwrap();
//     } else {
//         let cache_key = CacheKey::Kline(KlineCacheKey::new(params.exchange, params.symbol, params.interval));
//         let cache_value = engine_guard.get_cache_value(&cache_key, params.limit).await.unwrap();
//     }
//     (StatusCode::OK, Json(ApiResponse {
//         code: 0,
//         message: "success".to_string(),
//         data: Some(cache_value),
//     }))
// }


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




