use std::collections::HashMap;
use crate::{NodeOutputHandle, NodeMessageReceiver};
use event_center::EventPublisher;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;


pub struct BaseNodeState {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub node_output_handle: HashMap<String, NodeOutputHandle>,
    pub event_publisher: EventPublisher,
    pub enable_event_publish: bool,
    pub cancel_token: CancellationToken,
    pub message_receivers: Vec<NodeMessageReceiver>,
}

impl BaseNodeState {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String,
        event_publisher: EventPublisher,
    ) -> Self {
        Self { 
            strategy_id, 
            node_id, 
            node_name, 
            node_output_handle: HashMap::new(), 
            event_publisher,
            enable_event_publish: false, 
            cancel_token: CancellationToken::new(), 
            message_receivers: Vec::new() 
        }
    }
}






