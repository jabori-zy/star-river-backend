use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::cache::{KlineCacheKey, IndicatorCacheKey};
use crate::Event;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use types::indicator::Indicators;
use types::market::KlineSeries;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CommandEvent {
    KlineCacheManager(KlineCacheManagerCommand),
    IndicatorCacheManager(IndicatorCacheManagerCommand),
    IndicatorEngine(IndicatorEngineCommand),
    Database(DatabaseCommand),
    MarketDataEngine(MarketDataEngineCommand),
}

impl From<CommandEvent> for Event {
    fn from(event: CommandEvent) -> Self {
        Event::Command(event)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum KlineCacheManagerCommand {
    #[strum(serialize = "add-kline-cache-key")]
    AddKlineCacheKey(AddKlineCacheKeyParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKlineCacheKeyParams {
    pub strategy_id: i32,
    pub cache_key: KlineCacheKey,
    pub sender: String,
    pub timestamp:i64,
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorCacheManagerCommand {
    #[strum(serialize = "subscribe-indicator")]
    SubscribeIndicator(SubscribeIndicatorParams),
    #[strum(serialize = "get-subscribed-indicator")]
    GetSubscribedIndicator(GetSubscribedIndicatorParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeIndicatorParams {
    pub cache_key: IndicatorCacheKey,
    pub sender: String,
    pub timestamp:i64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct GetSubscribedIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}


// 指标引擎命令
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEngineCommand {
    #[strum(serialize = "calculate-indicator")]
    CalculateIndicator(CalculateIndicatorParams), // 计算指标
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
    pub kline_series: KlineSeries,
    pub sender: String,
    pub command_timestamp:i64,
    pub request_id: Uuid,
    pub batch_id: String,
}


// 数据库命令
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum DatabaseCommand {
    #[strum(serialize = "create-strategy")]
    // 创建策略
    CreateStrategy(CreateStrategyParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyParams {
    pub name: String,
    pub description: String,
}



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum MarketDataEngineCommand {
    #[strum(serialize = "subscribe-kline-stream")]
    SubscribeKlineStream(SubscribeKlineStreamParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineStreamParams {
    pub strategy_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}

