use serde::{Deserialize, Serialize};
use types::cache::IndicatorCacheKey;
use strum::Display;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineResponse {
    SubscribedIndicator(SubscribedIndicatorResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribedIndicatorResponse {
    pub indicator_cache_key_list: Vec<IndicatorCacheKey>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}