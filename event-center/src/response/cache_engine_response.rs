
use types::custom_type::{StrategyId, NodeId};
use types::cache::{CacheValue, Key};
use std::sync::Arc;
use std::collections::HashMap;
use crate::response::{Response, ResponseTrait};

#[derive(Debug)]
pub enum CacheEngineResponse {
    AddCacheKey(AddCacheKeyResponse),
    AddIndicatorCacheKey(AddIndicatorCacheKeyResponse),
    GetCacheData(GetCacheDataResponse),
    GetCacheDataMulti(GetCacheDataMultiResponse),
    GetCacheLength(GetCacheLengthResponse),
    GetCacheLengthMulti(GetCacheLengthMultiResponse),
}

impl ResponseTrait for CacheEngineResponse {
    fn code(&self) -> i32 {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.code,
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.code,
            CacheEngineResponse::GetCacheData(response) => response.code,
            CacheEngineResponse::GetCacheDataMulti(response) => response.code,
            CacheEngineResponse::GetCacheLength(response) => response.code,
            CacheEngineResponse::GetCacheLengthMulti(response) => response.code,
        }
    }

    fn message(&self) -> String {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.message.clone(),
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.message.clone(),
            CacheEngineResponse::GetCacheData(response) => response.message.clone(),
            CacheEngineResponse::GetCacheDataMulti(response) => response.message.clone(),
            CacheEngineResponse::GetCacheLength(response) => response.message.clone(),
            CacheEngineResponse::GetCacheLengthMulti(response) => response.message.clone(),
        }
    }

    fn response_timestamp(&self) -> i64 {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.response_timestamp,
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.response_timestamp,
            CacheEngineResponse::GetCacheData(response) => response.response_timestamp,
            CacheEngineResponse::GetCacheDataMulti(response) => response.response_timestamp,
            CacheEngineResponse::GetCacheLength(response) => response.response_timestamp,
            CacheEngineResponse::GetCacheLengthMulti(response) => response.response_timestamp,
        }
    }
}

impl From<CacheEngineResponse> for Response {
    fn from(response: CacheEngineResponse) -> Self {
        Response::CacheEngine(response)
    }
}

impl TryFrom<Response> for CacheEngineResponse {
    type Error = String;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::CacheEngine(response) => Ok(response),
            _ => Err("Invalid response type".to_string()),
        }
    }
}
#[derive(Debug)]
pub struct AddCacheKeyResponse {
    pub code: i32,
    pub message: String,
    pub cache_key: Key,
    pub response_timestamp: i64,
}

#[derive(Debug)]
pub struct GetCacheDataResponse {
    pub code: i32,
    pub message: String,
    pub cache_key: Key,
    pub cache_data: Vec<Arc<CacheValue>>,
    pub response_timestamp: i64
}

#[derive(Debug)]
pub struct GetCacheDataMultiResponse {
    pub code: i32,
    pub message: String,
    pub cache_data: HashMap<String, Vec<Vec<f64>>>,
    pub response_timestamp: i64,
}

#[derive(Debug)]
pub struct AddIndicatorCacheKeyResponse {
    pub code: i32,
    pub message: String,
    pub requested_strategy_id: StrategyId, // 请求的策略id
    pub requested_node_id: NodeId, // 请求的节点id
    pub indicator_cache_key: Key,
    pub response_timestamp: i64,
}

#[derive(Debug)]
pub struct GetCacheLengthResponse {
    pub code: i32,
    pub message: String,
    pub cache_key: Key,
    pub cache_length: u32,
    pub response_timestamp: i64,
}


#[derive(Debug)]
pub struct GetCacheLengthMultiResponse {
    pub code: i32,
    pub message: String,
    pub cache_length: HashMap<Key, u32>,
    pub response_timestamp: i64,
}
