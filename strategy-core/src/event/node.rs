
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::fmt::Debug;
use star_river_core::custom_type::{NodeId, NodeName};
use crate::node::node_handles::HandleId;


pub trait NodeEventTrait: Debug + Send + Sync + Clone {}






// 泛型事件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvent<T: Debug + Send + Sync + Clone> {
    #[serde(flatten)]
    pub node_event_base: NodeEventBase,
    #[serde(flatten)]
    pub payload: T,
}

impl<T: Debug + Send + Sync + Clone> NodeEvent<T> {
    pub fn new(from_node_id: String, from_node_name: String, from_node_handle_id: String, payload: T) -> Self {
        let node_event_base = NodeEventBase::new(from_node_id, from_node_name, from_node_handle_id);
        Self { node_event_base, payload }
    }
}

impl<T: Debug + Send + Sync + Clone> NodeEvent<T> {
    pub fn from_node_id(&self) -> &NodeId {
        &self.node_event_base.from_node_id
    }
    pub fn from_node_name(&self) -> &NodeName {
        &self.node_event_base.from_node_name
    }
    pub fn from_node_handle_id(&self) -> &HandleId {
        &self.node_event_base.from_node_handle_id
    }
}

// 使用 Deref 允许直接访问 payload 字段
impl<T: Debug + Send + Sync + Clone> Deref for NodeEvent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEventBase {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub datetime: chrono::DateTime<Utc>,
}

impl NodeEventBase {
    pub fn new(from_node_id: String, from_node_name: String, from_node_handle_id: String) -> Self {
        Self {
            from_node_id,
            from_node_name,
            from_node_handle_id,
            datetime: Utc::now(),
        }
    }
}
