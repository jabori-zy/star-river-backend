use crate::strategy_engine::strategy::Strategy;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::node_types::NodeMessageReceiver;

impl Strategy {
    pub async fn add_edge(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        from_node_id: &str,
        from_handle_id: &str,
        to_node_id: &str
    ) {
        if let (Some(&source), Some(&target)) = (
            node_indices.get(from_node_id),
            node_indices.get(to_node_id)
        ){
            
            tracing::debug!("添加边: {:?} -> {:?}, 源节点handle = {}", from_node_id, to_node_id, from_handle_id);
            // 先获取源节点的发送者
            let sender = graph.node_weight(source).unwrap().get_message_sender(from_handle_id.to_string()).await;
            
            tracing::debug!("{}: sender: {:?}", from_handle_id, sender);
            // 增加源节点的出口连接数
            graph.node_weight_mut(source).unwrap().add_output_handle_connect_count(from_handle_id.to_string()).await;
            // tracing::debug!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let message_receivers = target_node.get_message_receivers().await;
                tracing::debug!("{:?} 添加了一个接收者", target_node.get_node_name().await);
                target_node.add_message_receiver(NodeMessageReceiver::new(from_node_id.to_string(), receiver)).await;
                tracing::debug!("{}: 添加了一个接收者: {:?}", target_node.get_node_name().await, message_receivers);
                target_node.add_from_node_id(from_node_id.to_string()).await;
            }
            // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
            graph.add_edge(source, target, ());
        }
        

    }
}