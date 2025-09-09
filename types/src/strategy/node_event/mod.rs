pub mod backtest_node_event;
pub mod variable_event;

use crate::cache::key::IndicatorKey;
use crate::cache::key::KlineKey;
use crate::cache::CacheValue;
use crate::cache::KeyTrait;
use crate::custom_type::PlayIndex;
use crate::error::error_trait::{Language, StarRiverErrorTrait};
use crate::indicator::{Indicator, IndicatorConfig};
use crate::market::{Exchange, KlineInterval};
use crate::market::{Kline, KlineSeries};
use crate::order::virtual_order::VirtualOrder;
use crate::order::Order;
use crate::position::Position;
use crate::strategy::sys_varibale::SysVariable;
use backtest_node_event::futures_order_node_event::FuturesOrderNodeEvent;
use backtest_node_event::if_else_node_event::IfElseNodeEvent;
use backtest_node_event::kline_node_event::KlineNodeEvent;
use backtest_node_event::position_management_node_event::PositionManagementNodeEvent;
use backtest_node_event::variable_node_event::VariableNodeEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::Display;
use utils::get_utc8_timestamp_millis;
use variable_event::PositionNumberUpdateEvent;

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

    #[strum(serialize = "if_else_node")]
    #[serde(rename = "if_else_node")]
    IfElseNode(IfElseNodeEvent),
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

    #[serde(rename = "configId")]
    pub config_id: i32,

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

fn serialize_indicator_cache_key<'de, S>(
    indicator_cache_key: &IndicatorKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let indicator_cache_key_str = indicator_cache_key.get_key_str();
    serializer.serialize_str(&indicator_cache_key_str)
}

fn serialize_indicator_data<S>(
    indicator_data: &Vec<Arc<CacheValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(Some(indicator_data.len()))?;
    indicator_data
        .iter()
        .map(|indicator_value| {
            let json_value = indicator_value.to_json();
            seq.serialize_element(&json_value)
        })
        .collect::<Result<(), S::Error>>()?;
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
    KlinePlayFinished(KlinePlayFinishedEvent),   // k线播放完毕
    KlinePlay(KlinePlayEvent),                   // K线跳动(信号计数:根据这个值去请求缓存的下标)
    ExecuteOver(ExecuteOverEvent),               // 执行完毕
    RunningLog(StrategyRunningLogEvent),         // 回测条件匹配日志
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
fn serialize_cache_value_vec<S>(
    data: &Vec<Arc<CacheValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
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

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStateLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "nodeState")]
    pub node_state: String,

    #[serde(rename = "nodeStateAction")]
    pub node_state_action: String,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "errorCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

impl NodeStateLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Info,
            message,
            error_code: None,
            error_code_chain: None,
            timestamp: get_utc8_timestamp_millis(),
        }
    }

    pub fn error(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Error,
            message: error.get_error_message(Language::Chinese),
            error_code: Some(error.error_code().to_string()),
            error_code_chain: Some(error.error_code_chain()),
            timestamp: get_utc8_timestamp_millis(),
        }
    }
}

impl From<NodeStateLogEvent> for BacktestNodeEvent {
    fn from(event: NodeStateLogEvent) -> Self {
        BacktestNodeEvent::KlineNode(KlineNodeEvent::StartLog(event))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum StrategyRunningLogSource {
    #[strum(serialize = "node")]
    #[serde(rename = "Node")]
    Node,
    #[strum(serialize = "virtual_trading_system")]
    #[serde(rename = "VirtualTradingSystem")]
    VirtualTradingSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum StrategyRunningLogType {
    #[strum(serialize = "condition_match")]
    #[serde(rename = "ConditionMatch")]
    ConditionMatch,
}

// 策略运行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRunningLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "source")]
    pub source: StrategyRunningLogSource,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "logType")]
    pub log_type: StrategyRunningLogType,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "detail")]
    pub detail: serde_json::Value,

    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

impl StrategyRunningLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            source,
            log_level: LogLevel::Info,
            log_type,
            message,
            detail,
            error_code: None,
            error_code_chain: None,
            timestamp: get_utc8_timestamp_millis(),
        }
    }
}
