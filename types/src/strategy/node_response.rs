use crate::cache::CacheKey;


pub trait NodeResponseTrait {
    fn code(&self) -> i32;
    fn message(&self) -> String;
    fn response_timestamp(&self) -> i64;
}





#[derive(Debug)]
pub enum NodeResponse {
    Strategy(StrategyResponse),
}


impl NodeResponse {
    pub fn code(&self) -> i32 {
        match self {
            NodeResponse::Strategy(strategy_response) => strategy_response.code(),
        }
    }
    fn message(&self) -> String {
        match self {
            NodeResponse::Strategy(strategy_response) => strategy_response.message(),
        }
    }
    fn response_timestamp(&self) -> i64 {
        match self {
            NodeResponse::Strategy(strategy_response) => strategy_response.response_timestamp(),
        }
    }
}




#[derive(Debug)]
pub enum StrategyResponse {
    GetStrategyCacheKeys(GetStrategyCacheKeysResponse),
}

impl NodeResponseTrait for StrategyResponse {
    fn code(&self) -> i32 {
        match self {
            StrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.code,
        }
    }
    fn message(&self) -> String {
        match self {
            StrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.message.clone(),
        }
    }
    fn response_timestamp(&self) -> i64 {
        match self {
            StrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => get_strategy_cache_keys_response.response_timestamp,
        }
    }
}

impl From<StrategyResponse> for NodeResponse {
    fn from(response: StrategyResponse) -> Self {
        NodeResponse::Strategy(response)
    }
}

impl TryFrom<NodeResponse> for StrategyResponse {
    type Error = String;
    fn try_from(response: NodeResponse) -> Result<Self, Self::Error> {
        match response {    
            NodeResponse::Strategy(strategy_response) => Ok(strategy_response),
            _ => Err("try build strategy response from node response failed".to_string()),
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





