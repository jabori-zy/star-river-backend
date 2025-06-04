use crate::cache::CacheKey;


#[derive(Debug)]
pub enum NodeResponse {
    GetStrategyCacheKeys(GetStrategyCacheKeysResponse),
}


impl NodeResponse {
    pub fn code(&self) -> i32 {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.code,
        }
    }

    pub fn message(&self) -> String {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.message.clone(),
        }
    }

    pub fn response_timestamp(&self) -> i64 {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.response_timestamp,
        }
    }
}


#[derive(Debug)]
pub struct GetStrategyCacheKeysResponse {
    pub code: i32,
    pub message: String,
    pub cache_keys: Vec<CacheKey>,
    pub response_timestamp: i64,
}




