
use crate::market::{Kline, KlineSeries};
use crate::indicator::{IndicatorConfig, Indicator};
use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use strum::Display;
use crate::order::Order;
use crate::position::Position;
use crate::cache::CacheValue;
use std::sync::Arc;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum NodeMessage {
    #[strum(serialize = "kline_series")]
    #[serde(rename = "kline_series")]
    KlineSeries(KlineSeriesMessage),
    #[strum(serialize = "indicator")]
    #[serde(rename = "indicator")]
    Indicator(IndicatorMessage),
    #[strum(serialize = "signal")]
    #[serde(rename = "signal")]
    Signal(SignalMessage),
    #[strum(serialize = "order")]
    #[serde(rename = "order")]
    Order(OrderMessage),
    #[strum(serialize = "position")]
    #[serde(rename = "position")]
    Position(PositionMessage),
    #[strum(serialize = "variable")]
    #[serde(rename = "variable")]
    Variable(VariableMessage),
}

impl NodeMessage {
    pub fn as_indicator(&self) -> Option<&IndicatorMessage> {
        if let NodeMessage::Indicator(msg) = self {
            Some(msg)
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<&VariableMessage> {
        if let NodeMessage::Variable(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}


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

// 指标消息
#[derive(Debug, Serialize, Deserialize)]
pub struct IndicatorMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub indicator_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

impl Clone for IndicatorMessage {
    fn clone(&self) -> Self {
        IndicatorMessage {
            from_node_id: self.from_node_id.clone(),
            from_node_name: self.from_node_name.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            interval: self.interval.clone(),
            indicator: self.indicator.clone(),
            indicator_series: self.indicator_series.clone(),
            message_timestamp: self.message_timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    True,
    False,
}


// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    ConditionMatch,// 条件匹配
    OrderFilled, // 订单成交
    FetchKlineData(u32), // 拉取K线数据(信号计数:根据这个值去请求缓存的下标)
    KlinePlayFinished, // k线播放完毕
}


// 信号消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub signal_type: SignalType,
    pub message_timestamp: i64,
}


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
