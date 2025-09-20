use super::super::{EngineResponse, ResponseTrait};
use star_river_core::cache::{CacheValue, Key};
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::error::error_trait::StarRiverErrorTrait;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use star_river_core::system::DateTimeUtc;
use star_river_core::cache::CacheItem;

#[derive(Debug)]
pub enum CacheEngineResponse {
    AddCacheKey(AddCacheKeyResponse),
    AddIndicatorCacheKey(AddIndicatorCacheKeyResponse),
    GetCacheData(GetCacheDataResponse),
    GetCacheDataMulti(GetCacheDataMultiResponse),
    GetCacheLength(GetCacheLengthResponse),
    GetCacheLengthMulti(GetCacheLengthMultiResponse),
    UpdateCache(UpdateCacheResponse),
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
            CacheEngineResponse::UpdateCache(response) => response.success,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::AddIndicatorCacheKey(response) => {
                response.error.as_ref().unwrap().clone()
            }
            CacheEngineResponse::GetCacheData(response) => response.error.as_ref().unwrap().clone(),
            CacheEngineResponse::GetCacheDataMulti(response) => {
                response.error.as_ref().unwrap().clone()
            }
            CacheEngineResponse::GetCacheLength(response) => {
                response.error.as_ref().unwrap().clone()
            }
            CacheEngineResponse::GetCacheLengthMulti(response) => {
                response.error.as_ref().unwrap().clone()
            }
            CacheEngineResponse::UpdateCache(response) => {
                response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            CacheEngineResponse::AddCacheKey(response) => response.datetime,
            CacheEngineResponse::AddIndicatorCacheKey(response) => response.datetime,
            CacheEngineResponse::GetCacheData(response) => response.datetime,
            CacheEngineResponse::GetCacheDataMulti(response) => response.datetime,
            CacheEngineResponse::GetCacheLength(response) => response.datetime,
            CacheEngineResponse::GetCacheLengthMulti(response) => response.datetime,
            CacheEngineResponse::UpdateCache(response) => response.datetime,
        }
    }
}

impl From<CacheEngineResponse> for EngineResponse {
    fn from(response: CacheEngineResponse) -> Self {
        EngineResponse::CacheEngine(response)
    }
}

impl TryFrom<EngineResponse> for CacheEngineResponse {
    type Error = String;

    fn try_from(response: EngineResponse) -> Result<Self, Self::Error> {
        match response {
            EngineResponse::CacheEngine(response) => Ok(response),
            _ => Err("Invalid response type".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct AddCacheKeyResponse {
    pub success: bool,
    pub key: Key,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl AddCacheKeyResponse {
    pub fn success(key: Key) -> Self {
        Self {
            success: true,
            key,
            error: None,
            datetime: Utc::now()
        }
    }
}

// impl From<AddCacheKeyResponse> for Response {
//     fn from(response: AddCacheKeyResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::AddCacheKey(response))
//     }
// }

#[derive(Debug)]
pub struct GetCacheDataResponse {
    pub success: bool,
    pub key: Key,
    pub cache_data: Vec<Arc<CacheValue>>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetCacheDataResponse {
    pub fn success(key: Key, cache_data: Vec<Arc<CacheValue>>) -> Self {
        Self {
            success: true,
            key,
            cache_data,
            error: None,
            datetime: Utc::now()
        }
    }
}

// impl From<GetCacheDataResponse> for Response {
//     fn from(response: GetCacheDataResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::GetCacheData(response))
//     }
// }

#[derive(Debug)]
pub struct GetCacheDataMultiResponse {
    pub success: bool,
    pub cache_data: HashMap<String, Vec<Vec<f64>>>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetCacheDataMultiResponse {
    pub fn success(cache_data: HashMap<Key, Vec<Arc<CacheValue>>>) -> Self {
        Self {
            success: true,
            cache_data: cache_data
                .into_iter()
                .map(|(cache_key, data)| {
                    (
                        cache_key.get_key_str(),
                        data.into_iter()
                            .map(|cache_value| cache_value.to_list())
                            .collect(),
                    )
                })
                .collect(),
            error: None,
            datetime: Utc::now()
        }
    }
}

// impl From<GetCacheDataMultiResponse> for Response {
//     fn from(response: GetCacheDataMultiResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::GetCacheDataMulti(response))
//     }
// }

#[derive(Debug)]
pub struct AddIndicatorCacheKeyResponse {
    pub success: bool,
    pub requested_strategy_id: StrategyId, // 请求的策略id
    pub requested_node_id: NodeId,         // 请求的节点id
    pub indicator_key: Key,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl AddIndicatorCacheKeyResponse {
    pub fn success(
        requested_strategy_id: StrategyId,
        requested_node_id: NodeId,
        indicator_key: Key,
    ) -> Self {
        Self {
            success: true,
            requested_strategy_id,
            requested_node_id,
            indicator_key,
            error: None,
            datetime: Utc::now()
        }
    }
}

// impl From<AddIndicatorCacheKeyResponse> for Response {
//     fn from(response: AddIndicatorCacheKeyResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::AddIndicatorCacheKey(response))
//     }
// }

#[derive(Debug)]
pub struct GetCacheLengthResponse {
    pub success: bool,
    pub cache_key: Key,
    pub cache_length: u32,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetCacheLengthResponse {
    pub fn success(cache_key: Key, cache_length: u32) -> Self {
        Self {
            success: true,
            cache_key,
            cache_length,
            error: None,
            datetime: Utc::now()
        }
    }
}

// impl From<GetCacheLengthResponse> for Response {
//     fn from(response: GetCacheLengthResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::GetCacheLength(response))
//     }
// }

#[derive(Debug)]
pub struct GetCacheLengthMultiResponse {
    pub success: bool,
    pub cache_length: HashMap<Key, u32>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetCacheLengthMultiResponse {
    pub fn success(cache_length: HashMap<Key, u32>) -> Self {
        Self {
            success: true,
            cache_length,
            error: None,
            datetime: Utc::now()
        }
    }
}


// impl From<GetCacheLengthMultiResponse> for Response {
//     fn from(response: GetCacheLengthMultiResponse) -> Self {
//         Response::CacheEngine(CacheEngineResponse::GetCacheLengthMulti(response))
//     }
// }


#[derive(Debug)]
pub struct UpdateCacheResponse {
    pub success: bool,
    pub key: Key,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl UpdateCacheResponse {
    pub fn success(key: Key) -> Self {
        Self {
            success: true,
            key,
            error: None,
            datetime: Utc::now()
        }
    }
}


impl From<UpdateCacheResponse> for CacheEngineResponse {
    fn from(response: UpdateCacheResponse) -> Self {
        CacheEngineResponse::UpdateCache(response)
    }
}
