use crate::cache::Key;

#[derive(Debug)]
pub enum NodeResponse {
    GetStrategyCacheKeys(GetStrategyCacheKeysResponse),
    GetCurrentTime(GetCurrentTimeResponse),
}

impl NodeResponse {
    pub fn code(&self) -> i32 {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.code
            }
            NodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.code
            }
        }
    }

    pub fn message(&self) -> String {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.message.clone()
            }
            NodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.message.clone()
            }
        }
    }

    pub fn response_timestamp(&self) -> i64 {
        match self {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.response_timestamp
            }
            NodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.response_timestamp
            }
        }
    }
}

#[derive(Debug)]
pub struct GetStrategyCacheKeysResponse {
    pub code: i32,
    pub message: String,
    pub keys: Vec<Key>,
    pub response_timestamp: i64,
}

#[derive(Debug)]
pub struct GetCurrentTimeResponse {
    pub code: i32,
    pub message: String,
    pub current_time: i64,
    pub response_timestamp: i64,
}
