use serde::{Deserialize, Serialize};
use types::cache::IndicatorCacheKey;
use strum::Display;
use crate::Event;
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};
use types::indicator::{Indicators, IndicatorData};

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum ResponseEvent {
    CacheEngine(CacheEngineResponse),
    IndicatorEngine(IndicatorEngineResponse),
    MarketDataEngine(MarketDataEngineResponse),
}

impl From<ResponseEvent> for Event {
    fn from(event: ResponseEvent) -> Self {
        Event::Response(event)
    }
}


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


// 指标引擎响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorEngineResponse {
    // 计算指标完成
    CalculateIndicatorFinish(CalculateIndicatorResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateIndicatorResponse {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
    pub value: Box<dyn IndicatorData>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
    pub batch_id: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataEngineResponse {
    SubscribeKlineStreamSuccess(SubscribeKlineStreamSuccessResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineStreamSuccessResponse {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

