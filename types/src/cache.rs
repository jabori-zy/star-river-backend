use crate::market::Exchange;
use serde::{Deserialize, Serialize};
use crate::indicator::Indicators;
use crate::market::KlineInterval;
use std::hash::Hash;
use std::fmt::Debug;

pub trait CacheKey: Debug + Clone + Hash + Eq + PartialEq + Serialize {}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlineCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl CacheKey for KlineCacheKey {}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndicatorCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
}

impl CacheKey for IndicatorCacheKey {}




// 缓存管理器



