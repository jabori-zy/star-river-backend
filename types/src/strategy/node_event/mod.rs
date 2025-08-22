pub mod variable_event;
pub mod backtest_node_event;


use crate::market::{Kline, KlineSeries};
use crate::indicator::{IndicatorConfig, Indicator};
use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use strum::Display;
use crate::order::Order;
use crate::position::Position;
use crate::cache::CacheValue;
use crate::cache::key::KlineKey;
use std::sync::Arc;
use variable_event::PositionNumberUpdateEvent;
use crate::order::virtual_order::VirtualOrder;
use backtest_node_event::kline_node_event::KlineNodeEvent;
use crate::cache::key::IndicatorKey;
use crate::cache::KeyTrait;
use backtest_node_event::futures_order_node_event::FuturesOrderNodeEvent;
use backtest_node_event::position_management_node_event::PositionManagementNodeEvent;
use crate::strategy::sys_varibale::SysVariable;
use backtest_node_event::variable_node_event::VariableNodeEvent;
use crate::custom_type::PlayIndex;





#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "node_type")]
pub enum BacktestNodeEvent {
    #[strum(serialize = "kline_series")]
    #[serde(rename = "kline_series")]
    KlineSeries(KlineSeriesMessage),

    #[strum(serialize = "indicator")]
    #[serde(rename = "indicator")]
    IndicatorNode(IndicatorNodeEvent),

    #[strum(serialize = "signal")]
    #[serde(rename = "signal")]
    Signal(SignalEvent),

    #[strum(serialize = "order")]
    #[serde(rename = "order")]
    Order(OrderEvent),

    #[strum(serialize = "position")]
    #[serde(rename = "position")]
    Position(PositionEvent),

    #[strum(serialize = "variable")]
    #[serde(rename = "variable")]
    Variable(VariableNodeEvent),

    #[strum(serialize = "kline-node")]
    #[serde(rename = "kline-node")]
    KlineNode(KlineNodeEvent), // 回测K线更新(缓存index, K线) 回测k线更新

    #[strum(serialize = "futures_order_node")]
    #[serde(rename = "futures_order_node")]
    FuturesOrderNode(FuturesOrderNodeEvent),

    #[strum(serialize = "position_management_node")]
    #[serde(rename = "position_management_node")]
    PositionManagementNode(PositionManagementNodeEvent),


}


// k线系列消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesMessage {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    #[serde(serialize_with = "serialize_cache_value_vec")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub kline_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorNodeEvent {
    #[strum(serialize = "indicator_update")]
    #[serde(rename = "indicator_update")]
    LiveIndicatorUpdate(LiveIndicatorUpdateEvent), // 实盘指标更新
    IndicatorUpdate(IndicatorUpdateEvent), // 回测指标更新
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
    #[serde(serialize_with = "serialize_cache_value_vec")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub indicator_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndicatorUpdateEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromNodeHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "exchange")]
    pub exchange: Exchange,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "interval")]
    pub interval: KlineInterval,

    #[serde(rename = "indicatorId")]
    pub indicator_id: i32,

    #[serde(rename = "indicatorConfig")]
    pub indicator_config: IndicatorConfig,

    #[serde(rename = "indicatorKey")]
    #[serde(serialize_with = "serialize_indicator_cache_key")]
    pub indicator_key: IndicatorKey,

    #[serde(rename = "indicatorSeries")]
    #[serde(serialize_with = "serialize_indicator_data")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub indicator_series: Vec<Arc<CacheValue>>,

    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(rename = "timestamp")] 
    pub timestamp: i64,
}

fn serialize_indicator_cache_key<'de, S>(indicator_cache_key: &IndicatorKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let indicator_cache_key_str = indicator_cache_key.get_key_str();
    serializer.serialize_str(&indicator_cache_key_str)
}

fn serialize_indicator_data<S>(indicator_data: &Vec<Arc<CacheValue>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    
    let mut seq = serializer.serialize_seq(Some(indicator_data.len()))?;
    indicator_data.iter().map(|indicator_value| {
        let json_value = indicator_value.to_json();
        seq.serialize_element(&json_value)
    }).collect::<Result<(), S::Error>>()?;
    seq.end()
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
pub enum OrderEvent {
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
#[serde(tag = "event_type")]
pub enum PositionEvent {
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
    pub variable_config_id: i32, // 变量配置id
    pub variable: SysVariable,
    pub variable_value: f64,
    pub message_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum VariableEvent {
    #[strum(serialize = "position-number-updated")]
    #[serde(rename = "position-number-updated")]
    PositionNumberUpdate(PositionNumberUpdateEvent), // 仓位数量更新
}

impl VariableEvent {
    pub fn get_from_node_id(&self) -> String {
        match self {
            VariableEvent::PositionNumberUpdate(event) => event.from_node_id.clone(),
        }
    }

}









#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalEvent {
    LiveConditionMatch(LiveConditionMatchEvent), // 实盘条件匹配
    BacktestConditionMatch(BacktestConditionMatchEvent), // 回测条件匹配
    BacktestConditionNotMatch(BacktestConditionNotMatchEvent), // 回测条件不匹配
    KlinePlayFinished(KlinePlayFinishedEvent), // k线播放完毕
    KlinePlay(KlinePlayEvent), // K线跳动(信号计数:根据这个值去请求缓存的下标)
    ExecuteOver(ExecuteOverEvent), // 执行完毕
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConditionMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub message_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConditionMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConditionNotMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: i32,
    pub message_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayFinishedEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: i32,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}

// 通用的序列化函数
fn serialize_cache_value_vec<S>(data: &Vec<Arc<CacheValue>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    
    let mut seq = serializer.serialize_seq(Some(data.len()))?;
    for item in data {
        let json_value = item.to_json();
        seq.serialize_element(&json_value)?;
    }
    seq.end()
}

// 通用的反序列化函数
fn deserialize_cache_value_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<CacheValue>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;
    
    // 这里我们简单地跳过反序列化，返回空向量
    // 在实际应用中，你可能需要根据具体需求来实现反序列化逻辑
    let _: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    Ok(Vec::new())
}
