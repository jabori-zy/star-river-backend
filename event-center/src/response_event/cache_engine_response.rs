use serde::{Deserialize, Serialize};
use types::cache::IndicatorCacheKey;
use strum::Display;
use uuid::Uuid;
use types::new_cache::{CacheValue, CacheKey};



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineResponse {
    SubscribedIndicator(SubscribedIndicatorResponse),
    GetCacheData(GetCacheDataResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribedIndicatorResponse {
    pub indicator_cache_key_list: Vec<IndicatorCacheKey>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheDataResponse {
    pub cache_key: CacheKey,
    pub cache_data: Vec<CacheValue>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

