use std::error::Error;

use types::custom_type::{StrategyId, NodeId};
use types::cache::{CacheValue, Key};
use std::sync::Arc;
use std::collections::HashMap;
use crate::response::{Response, ResponseTrait};
use utils::get_utc8_timestamp;

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
    fn success(&self) -> bool {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.success,
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.success,
            CacheEngineResponse::GetCacheData(response) => response.success,
            CacheEngineResponse::GetCacheDataMulti(response) => response.success,
            CacheEngineResponse::GetCacheLength(response) => response.success,
            CacheEngineResponse::GetCacheLengthMulti(response) => response.success,
        }
    }
    

    fn error(&self) -> Arc<dyn Error + Send + Sync + 'static> {
        match self {
            CacheEngineResponse::AddCacheKey(response) =>  response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::GetCacheData(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::GetCacheDataMulti(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::GetCacheLength(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::GetCacheLengthMulti(response) => response.error.as_ref().unwrap().clone(),
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
    pub success: bool,
    pub key: Key,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}


impl AddCacheKeyResponse {
    pub fn success(key: Key) -> Self {
        Self {
            success: true,
            key,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<AddCacheKeyResponse> for Response {
    fn from(response: AddCacheKeyResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::AddCacheKey(response))
    }
}





#[derive(Debug)]
pub struct GetCacheDataResponse {
    pub success: bool,
    pub key: Key,
    pub cache_data: Vec<Arc<CacheValue>>,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}


impl GetCacheDataResponse {
    pub fn success(key: Key, cache_data: Vec<Arc<CacheValue>>) -> Self {
        Self {
            success: true,
            key,
            cache_data,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<GetCacheDataResponse> for Response {
    fn from(response: GetCacheDataResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::GetCacheData(response))
    }
}




#[derive(Debug)]
pub struct GetCacheDataMultiResponse {
    pub success: bool,
    pub cache_data: HashMap<String, Vec<Vec<f64>>>,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}


impl GetCacheDataMultiResponse {
    pub fn success(cache_data: HashMap<Key, Vec<Arc<CacheValue>>>) -> Self {
        Self {
            success: true,
            cache_data: cache_data.into_iter().map(|(cache_key, data)| (cache_key.get_key(), data.into_iter().map(|cache_value| cache_value.to_list()).collect())).collect(),
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<GetCacheDataMultiResponse> for Response {
    fn from(response: GetCacheDataMultiResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::GetCacheDataMulti(response))
    }
}









#[derive(Debug)]
pub struct AddIndicatorCacheKeyResponse {
    pub success: bool,  
    pub requested_strategy_id: StrategyId, // 请求的策略id
    pub requested_node_id: NodeId, // 请求的节点id
    pub indicator_key: Key,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}

impl AddIndicatorCacheKeyResponse {
    pub fn success(requested_strategy_id: StrategyId, requested_node_id: NodeId, indicator_key: Key) -> Self {
        Self {
            success: true,
            requested_strategy_id,
            requested_node_id,
            indicator_key,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}


impl From<AddIndicatorCacheKeyResponse> for Response {
    fn from(response: AddIndicatorCacheKeyResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::AddIndicatorCacheKey(response))
    }
}



#[derive(Debug)]
pub struct GetCacheLengthResponse {
    pub success: bool,
    pub cache_key: Key,
    pub cache_length: u32,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}


impl GetCacheLengthResponse {

    pub fn success(cache_key: Key, cache_length: u32) -> Self {
        Self {
            success: true,
            cache_key,
            cache_length,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<GetCacheLengthResponse> for Response {

    fn from(response: GetCacheLengthResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::GetCacheLength(response))
    }
}


#[derive(Debug)]
pub struct GetCacheLengthMultiResponse {
    pub success: bool,
    pub cache_length: HashMap<Key, u32>,
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}


impl GetCacheLengthMultiResponse {
    pub fn success(cache_length: HashMap<Key, u32>) -> Self {
        Self {
            success: true,
            cache_length,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<GetCacheLengthMultiResponse> for Response {
    fn from(response: GetCacheLengthMultiResponse) -> Self {
        Response::CacheEngine(CacheEngineResponse::GetCacheLengthMulti(response))
    }
}