use std::{fmt::Debug, ops::Deref};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{NodeId, NodeName};

use crate::node::node_handles::HandleId;

pub trait NodeEventTrait: Debug + Send + Sync + Clone + 'static {
    fn node_id(&self) -> &NodeId;
    fn node_name(&self) -> &NodeName;
    fn output_handle_id(&self) -> &HandleId;
}

// 泛型事件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvent<T: Debug + Send + Sync + Clone> {
    #[serde(flatten)]
    pub node_event_base: NodeEventBase,
    #[serde(flatten)]
    pub payload: T,
}

impl<T: Debug + Send + Sync + Clone> NodeEvent<T> {
    pub fn new(node_id: NodeId, node_name: NodeName, output_handle_id: HandleId, payload: T) -> Self {
        let node_event_base = NodeEventBase::new(node_id, node_name, output_handle_id);
        Self { node_event_base, payload }
    }
}

impl<T: Debug + Send + Sync + Clone> NodeEvent<T> {
    pub fn node_id(&self) -> &NodeId {
        &self.node_event_base.node_id
    }
    pub fn node_name(&self) -> &NodeName {
        &self.node_event_base.node_name
    }
    pub fn output_handle_id(&self) -> &HandleId {
        &self.node_event_base.output_handle_id
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
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub output_handle_id: HandleId,
    pub datetime: chrono::DateTime<Utc>,
}

impl NodeEventBase {
    pub fn new(node_id: NodeId, node_name: NodeName, output_handle_id: HandleId) -> Self {
        Self {
            node_id,
            node_name,
            output_handle_id,
            datetime: Utc::now(),
        }
    }
}
