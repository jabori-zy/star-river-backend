pub mod common_event;
pub mod futures_order_node_event;
pub mod if_else_node_event;
pub mod indicator_node_event;
pub mod kline_node_event;
pub mod position_node_event;
pub mod start_node_event;
pub mod variable_node_event;

pub use common_event::CommonEvent;
pub use futures_order_node_event::FuturesOrderNodeEvent;
pub use if_else_node_event::IfElseNodeEvent;
pub use indicator_node_event::IndicatorNodeEvent;
pub use kline_node_event::KlineNodeEvent;
pub use position_node_event::PositionManagementNodeEvent;
use star_river_core::market::{Kline, QuantData};
pub use start_node_event::StartNodeEvent;
pub use variable_node_event::VariableNodeEvent;

use derive_more::From;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "node_type")]
pub enum BacktestNodeEvent {
    #[strum(serialize = "start_node")]
    #[serde(rename = "start_node")]
    StartNode(StartNodeEvent),

    #[strum(serialize = "indicator_node")]
    #[serde(rename = "indicator_node")]
    IndicatorNode(IndicatorNodeEvent),

    #[strum(serialize = "common")]
    #[serde(rename = "common")]
    Common(CommonEvent),

    #[strum(serialize = "variable_node")]
    #[serde(rename = "variable_node")]
    VariableNode(VariableNodeEvent),

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

// 通用的序列化函数
#[allow(dead_code)]
fn serialize_cache_value_vec<S>(data: &Vec<Kline>, serializer: S) -> Result<S::Ok, S::Error>
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
#[allow(dead_code)]
fn deserialize_cache_value_vec<'de, D>(deserializer: D) -> Result<Vec<Kline>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    // 这里我们简单地跳过反序列化，返回空向量
    // 在实际应用中，你可能需要根据具体需求来实现反序列化逻辑
    let _: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    Ok(Vec::new())
}
