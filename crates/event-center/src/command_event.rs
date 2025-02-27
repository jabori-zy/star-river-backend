use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::cache::{KlineCacheKey, IndicatorCacheKey};
use crate::Event;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CommandEvent {
    KlineCacheManager(KlineCacheManagerCommand),
    IndicatorCacheManager(IndicatorCacheManagerCommand),
}

impl From<CommandEvent> for Event {
    fn from(event: CommandEvent) -> Self {
        Event::Command(event)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum KlineCacheManagerCommand {
    #[strum(serialize = "subscribe-kline")]
    SubscribeKline(SubscribeKlineParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineParams {
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




