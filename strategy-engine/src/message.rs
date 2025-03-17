use chrono::{DateTime, Utc};
use tokio::sync::broadcast::error::SendError;
use std::error::Error;
use async_trait::async_trait;
use types::market::KlineSeries;
use types::indicator::{Indicators, IndicatorData};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};


// #[derive(Debug, Clone)]
// pub struct NodeMessage {
//     pub from_node_id: Uuid,
//     pub from_node_name: String,
//     pub message_type: MessageType,
//     pub message: Message,
//     pub batch_id: String,
//     pub timestamp: i64,
// }

#[derive(Debug, Clone)]
pub enum NodeMessage {
    KlineSeries(KlineSeriesMessage),
    Indicator(IndicatorMessage),
    Signal(SignalMessage),
}

impl NodeMessage {
    pub fn as_indicator(&self) -> Option<&IndicatorMessage> {
        if let NodeMessage::Indicator(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}



// k线系列消息
#[derive(Debug, Clone)]
pub struct KlineSeriesMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub batch_id: String,
    pub message_timestamp: i64,
}

// 指标消息
#[derive(Debug)]
pub struct IndicatorMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
    pub indicator_data: Box<dyn IndicatorData>,
    pub batch_id: String,
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
            indicator_data: self.indicator_data.clone_box(),
            batch_id: self.batch_id.clone(),
            message_timestamp: self.message_timestamp,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Signal {
    True,
    False,
}

// 信号消息
#[derive(Debug, Clone)]
pub struct SignalMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle: String,
    pub signal: Signal,
    pub message_timestamp: i64,
}
