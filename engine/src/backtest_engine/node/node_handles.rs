// ============================================================================
// 标准库导入
// ============================================================================

use std::fmt::Debug;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::custom_type::{HandleId, NodeId};
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct NodeInputHandle {
    // 来自哪个节点
    pub from_node_id: String,
    pub from_handle_id: String,
    pub input_handle_id: HandleId, // 对应的input_handle_id
    pub receiver: broadcast::Receiver<BacktestNodeEvent>,
}

impl NodeInputHandle {
    pub fn new(
        from_node_id: String,
        from_handle_id: String,
        input_handle_id: HandleId,
        receiver: broadcast::Receiver<BacktestNodeEvent>,
    ) -> Self {
        Self {
            from_node_id,
            from_handle_id,
            input_handle_id,
            receiver,
        }
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<BacktestNodeEvent> {
        self.receiver.resubscribe()
    }
}

impl Clone for NodeInputHandle {
    fn clone(&self) -> Self {
        Self {
            from_node_id: self.from_node_id.clone(),
            from_handle_id: self.from_handle_id.clone(),
            input_handle_id: self.input_handle_id.clone(),
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
pub struct NodeOutputHandle {
    node_id: NodeId,
    output_handle_id: HandleId,
    node_event_sender: broadcast::Sender<BacktestNodeEvent>,
    subscriber: Vec<String>, // 订阅者（id）
}

impl NodeOutputHandle {
    pub fn new(node_id: NodeId, output_handle_id: HandleId, node_event_sender: broadcast::Sender<BacktestNodeEvent>) -> Self {
        Self {
            node_id,
            output_handle_id,
            node_event_sender,
            subscriber: vec![],
        }
    }

    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    pub fn subscriber(&self) -> Vec<String> {
        self.subscriber.clone()
    }

    pub fn output_handle_id(&self) -> &HandleId {
        &self.output_handle_id
    }

    pub fn send(&self, event: BacktestNodeEvent) -> Result<usize, String> {
        if self.node_event_sender.receiver_count() > 0 {
            self.node_event_sender
                .send(event)
                .map_err(|e| format!("节点{}的出口{}发送消息失败: {}", self.node_id, self.output_handle_id, e))
        } else {
            // 如果connect_count为1(默认的一个是连接到策略的)，则不发送消息
            Err(format!(
                "output handle have no connection, node_id:{}, output_handle_id:{}",
                self.node_id, self.output_handle_id
            ))
        }
    }

    pub fn subscribe(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        self.subscriber.push(subscriber_id);
        let receiver = self.node_event_sender.subscribe();
        receiver
    }

    pub fn receiver_count(&self) -> usize {
        self.node_event_sender.receiver_count()
    }

    pub fn is_connected(&self) -> bool {
        self.receiver_count() > 0
    }
}

impl std::fmt::Display for NodeOutputHandle {
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

impl Debug for NodeOutputHandle {
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
