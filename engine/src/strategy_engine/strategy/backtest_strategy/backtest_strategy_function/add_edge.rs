use crate::strategy_engine::node::BacktestNodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::node_types::NodeInputHandle;
use super::BacktestStrategyFunction;

impl BacktestStrategyFunction {
    pub async fn add_edge(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        from_node_id: &str,
        from_handle_id: &str,
        to_node_id: &str,
        to_handle_id: &str
    ) {
        if let (Some(&source), Some(&target)) = (
            node_indices.get(from_node_id),
            node_indices.get(to_node_id)
        ){
            
            tracing::debug!("添加边: {:?} -> {:?}, 源节点handle = {}", from_node_id, to_node_id, from_handle_id);
            let from_node_handles = graph.node_weight(source).unwrap().get_all_output_handles().await;
            tracing::debug!("from_node_handles: {:?}", from_node_handles);
            // 先获取源节点的output_handle
            let from_node_output_handle = graph.node_weight(source).unwrap().get_output_handle(&from_handle_id.to_string()).await;
            
            tracing::debug!("{}: from_node_output_handle: {:?}", from_handle_id, from_node_output_handle);
            // 增加源节点的出口连接数
            graph.node_weight_mut(source).unwrap().add_output_handle_connect_count(&from_handle_id.to_string()).await;
            // tracing::debug!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = from_node_output_handle.subscribe();
                // 获取接收者数量
                
                // tracing::debug!("{:?} 添加了一个接收者", target_node.get_node_name().await);
                let node_message_receiver = NodeInputHandle::new(
                    from_node_id.to_string(), 
                    from_handle_id.to_string(),
                    to_handle_id.to_string(), 
                    receiver);
                target_node.add_message_receiver(node_message_receiver).await;
                let message_receivers = target_node.get_node_event_receivers().await;
                tracing::debug!("{}: 添加了一个接收者: {:?}", target_node.get_node_name().await, message_receivers);
                target_node.add_from_node_id(from_node_id.to_string()).await;
            }
            // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
            graph.add_edge(source, target, ());
        }
        

    }
}