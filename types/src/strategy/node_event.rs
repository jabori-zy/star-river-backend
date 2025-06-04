
use crate::market::{Kline, KlineSeries};
use crate::indicator::{IndicatorConfig, Indicator};
use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use strum::Display;
use crate::order::Order;
use crate::position::Position;
use crate::cache::CacheValue;
use crate::cache::cache_key::BacktestKlineCacheKey;
use std::sync::Arc;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum NodeEvent {
    #[strum(serialize = "kline_series")]
    #[serde(rename = "kline_series")]
    KlineSeries(KlineSeriesMessage),
    #[strum(serialize = "indicator")]
    #[serde(rename = "indicator")]
    Indicator(IndicatorEvent),
    #[strum(serialize = "signal")]
    #[serde(rename = "signal")]
    Signal(SignalEvent),
    #[strum(serialize = "order")]
    #[serde(rename = "order")]
    Order(OrderMessage),
    #[strum(serialize = "position")]
    #[serde(rename = "position")]
    Position(PositionMessage),
    #[strum(serialize = "variable")]
    #[serde(rename = "variable")]
    Variable(VariableMessage),
    #[strum(serialize = "backtest_kline_update")]
    #[serde(rename = "backtest_kline_update")]
    BacktestKline(BacktestKlineUpdateEvent), // 回测K线更新(缓存index, K线) 回测k线更新
    // #[strum(serialize = "backtest_signal")]
    // #[serde(rename = "backtest_signal")]
    // BacktestSignal(BacktestSignalEvent), // 回测信号
}

// impl NodeEvent {
//     pub fn as_indicator(&self) -> Option<&LiveIndicatorUpdateEvent> {
//         if let NodeEvent::Indicator(msg) = self {
//             Some(msg)
//         } else {
//             None
//         }
//     }

//     pub fn as_variable(&self) -> Option<&VariableMessage> {
//         if let NodeEvent::Variable(msg) = self {
//             Some(msg)
//         } else {
//             None
//         }
//     }
// }


// k线系列消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEvent {
    #[strum(serialize = "indicator_update")]
    #[serde(rename = "indicator_update")]
    LiveIndicatorUpdate(LiveIndicatorUpdateEvent), // 实盘指标更新
    BacktestIndicatorUpdate(BacktestIndicatorUpdateEvent), // 回测指标更新
    IndicatorUpdateError,
}

// 指标消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveIndicatorUpdateEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
    pub indicator_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BacktestIndicatorUpdateEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
    pub indicator_series: Vec<Arc<CacheValue>>,
    pub kline_cache_index: u32,
    pub message_timestamp: i64,
}



// 信号类型
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum SignalType {
//     ConditionMatch,// 条件匹配
//     OrderFilled, // 订单成交
//     KlinePlayFinished, // k线播放完毕
//     // KlineTick(u32), // K线跳动(信号计数:根据这个值去请求缓存的下标)
// }


// 信号消息
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SignalMessage {
//     pub from_node_id: String,
//     pub from_node_name: String,
//     pub from_node_handle_id: String,
//     pub signal_type: SignalType,
//     pub message_timestamp: i64,
// }


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum OrderMessage {
    #[strum(serialize = "order-created")]
    #[serde(rename = "order-created")]
    OrderCreated(Order),
    #[strum(serialize = "order-updated")]
    #[serde(rename = "order-updated")]
    OrderUpdated(Order),
    #[strum(serialize = "order-canceled")]
    #[serde(rename = "order-canceled")]
    OrderCanceled(Order),
    #[strum(serialize = "order-filled")]
    #[serde(rename = "order-filled")]
    OrderFilled(Order),
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum PositionMessage {
    #[strum(serialize = "position-updated")]
    #[serde(rename = "position-updated")]
    PositionUpdated(Position),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub struct VariableMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub variable: String,
    pub variable_value: f64,
    pub message_timestamp: i64,

    
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestKlineUpdateEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub kline_cache_index: u32,
    pub kline_cache_key: BacktestKlineCacheKey,
    pub kline: Vec<f64>,
    pub message_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalEvent {
    ConditionMatch(ConditionMatchEvent), // 实盘条件匹配
    KlinePlayFinished(KlinePlayFinishedEvent), // k线播放完毕
    KlineTick(KlineTickEvent), // K线跳动(信号计数:根据这个值去请求缓存的下标)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub message_timestamp: i64,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineTickEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub signal_index: u32,
    pub message_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayFinishedEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub signal_index: u32,
    pub message_timestamp: i64,
}