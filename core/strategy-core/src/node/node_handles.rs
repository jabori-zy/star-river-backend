// std
use std::{fmt::Debug, sync::Arc};

use snafu::IntoError;
// workspace crate
use star_river_core::custom_type::{NodeId, NodeName};
// third-party
use tokio::sync::broadcast;

use crate::error::{NodeError, node_error::NodeEventSendFailedSnafu};

pub type HandleId = String;

#[derive(Debug)]
pub struct NodeInputHandle<E: Clone> {
    // 来自哪个节点
    pub from_node_id: String,
    pub from_handle_id: String,
    pub input_handle_id: HandleId, // 对应的input_handle_id
    pub config_id: i32,
    pub receiver: broadcast::Receiver<E>,
}

impl<E: Clone> NodeInputHandle<E> {
    pub fn new(from_node_id: String, from_handle_id: String, input_handle_id: HandleId, config_id: i32, receiver: broadcast::Receiver<E>) -> Self {
        Self {
            from_node_id,
            from_handle_id,
            input_handle_id,
            config_id,
            receiver,
        }
    }

    pub fn receiver(&self) -> broadcast::Receiver<E> {
        self.receiver.resubscribe()
    }
}

impl<E: Clone> Clone for NodeInputHandle<E> {
    fn clone(&self) -> Self {
        Self {
            from_node_id: self.from_node_id.clone(),
            from_handle_id: self.from_handle_id.clone(),
            input_handle_id: self.input_handle_id.clone(),
            config_id: self.config_id,
            receiver: self.receiver.resubscribe(),
        }
    }
}

// #[derive(Clone, Debug)]
// pub struct Edge {
//     pub id: String,
//     pub source: NodeType,
//     pub target: NodeType,
// }

#[derive(Clone)]
pub struct NodeOutputHandle<E> {
    node_id: NodeId,
    node_name: NodeName,
    is_default: bool,
    config_id: i32,
    output_handle_id: HandleId,
    node_event_sender: broadcast::Sender<E>,
    subscriber: Vec<String>, // 订阅者（id）
}

impl<E> NodeOutputHandle<E> {
    pub fn new(
        node_id: NodeId,
        node_name: NodeName,
        is_default: bool,
        config_id: i32,
        output_handle_id: HandleId,
        node_event_sender: broadcast::Sender<E>,
    ) -> Self {
        Self {
            node_id,
            node_name,
            is_default,
            config_id,
            output_handle_id,
            node_event_sender,
            subscriber: vec![],
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }

    pub fn node_name(&self) -> NodeName {
        self.node_name.clone()
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn subscriber(&self) -> Vec<String> {
        self.subscriber.clone()
    }

    pub fn config_id(&self) -> i32 {
        self.config_id
    }

    pub fn output_handle_id(&self) -> &HandleId {
        &self.output_handle_id
    }

    pub fn send(&self, event: E) -> Result<(), NodeError>
    where
        E: Debug + Send + Sync + 'static,
    {
        let result = self.node_event_sender.send(event).map_err(|e| {
            NodeEventSendFailedSnafu {
                node_name: self.node_name().clone(),
                handle_id: self.output_handle_id.clone(),
            }
            .into_error(Arc::new(e))
        });
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn subscribe(&mut self, subscriber_id: String) -> (i32, broadcast::Receiver<E>) {
        self.subscriber.push(subscriber_id);
        let receiver = self.node_event_sender.subscribe();
        (self.config_id, receiver)
    }

    pub fn receiver_count(&self) -> usize {
        self.node_event_sender.receiver_count()
    }

    pub fn is_connected(&self) -> bool {
        self.receiver_count() > 0
    }
}

impl<E> std::fmt::Display for NodeOutputHandle<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeOutputHandle(node_id: {}, output_handle_id: {}, receiver_count: {}, subscriber: {:?})",
            self.node_id,
            self.output_handle_id,
            self.receiver_count(),
            self.subscriber
        )
    }
}

impl<E> Debug for NodeOutputHandle<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeOutputHandle(node_id: {}, output_handle_id: {}, receiver_count: {}, subscriber: {:?})",
            self.node_id,
            self.output_handle_id,
            self.receiver_count(),
            self.subscriber
        )
    }
}
