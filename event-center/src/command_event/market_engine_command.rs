
use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum MarketEngineCommand {
    #[strum(serialize = "subscribe-kline-stream")]
    SubscribeKlineStream(SubscribeKlineStreamParams),
    #[strum(serialize = "unsubscribe-kline-stream")]
    UnsubscribeKlineStream(UnsubscribeKlineStreamParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineStreamParams {
    pub strategy_id: i64,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  UnsubscribeKlineStreamParams {
    pub strategy_id: i64,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}