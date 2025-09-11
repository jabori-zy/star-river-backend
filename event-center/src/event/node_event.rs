pub mod backtest_node_event;

pub use backtest_node_event::BacktestNodeEvent;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

// 泛型事件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvent<T> {
    #[serde(flatten)]
    pub node_event_base: NodeEventBase,
    #[serde(flatten)]
    pub payload: T,
}

impl<T> NodeEvent<T> {
    pub fn new(
        from_node_id: String,
        from_node_name: String,
        from_node_handle_id: String,
        payload: T,
    ) -> Self {
        let node_event_base = NodeEventBase::new(from_node_id, from_node_name, from_node_handle_id);
        Self {
            node_event_base,
            payload,
        }
    }
}

impl<T> NodeEventTrait for NodeEvent<T> {
    fn node_event_base(&self) -> &NodeEventBase {
        &self.node_event_base
    }
}

// 使用 Deref 允许直接访问 payload 字段
impl<T> Deref for NodeEvent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

pub trait NodeEventTrait {
    fn node_event_base(&self) -> &NodeEventBase;
    fn from_node_id(&self) -> &String {
        &self.node_event_base().from_node_id
    }
    fn from_node_name(&self) -> &String {
        &self.node_event_base().from_node_name
    }
    fn from_node_handle_id(&self) -> &String {
        &self.node_event_base().from_node_handle_id
    }
    fn datetime(&self) -> DateTime<FixedOffset> {
        self.node_event_base().datetime
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEventBase {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,
    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,
    #[serde(rename = "fromNodeHandleId")]
    pub from_node_handle_id: String,
    #[serde(rename = "datetime")]
    pub datetime: DateTime<FixedOffset>,
}

impl NodeEventBase {
    pub fn new(from_node_id: String, from_node_name: String, from_node_handle_id: String) -> Self {
        Self {
            from_node_id,
            from_node_name,
            from_node_handle_id,
            datetime: utils::get_utc8_datetime(),
        }
    }
}
