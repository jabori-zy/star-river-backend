mod add_edge;
mod add_node;
mod add_start_node;
mod add_live_data_node;
mod add_if_else_node;
mod add_indicator_node;
mod add_order_node;
mod add_position_node;
mod add_get_variable_node;
pub mod sys_variable_function;
use crate::strategy_engine::node::NodeTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use petgraph::{Graph, Directed};


pub struct LiveStrategyFunction;


impl LiveStrategyFunction {
    // 将所有节点的output_handle添加到策略中
    pub async fn add_node_output_handle(graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>) -> Vec<NodeOutputHandle> {
        tracing::debug!("添加所有节点节点的输出句柄");
        let mut strategy_output_handles = Vec::new();
        // 先将所有的连接数+1
        for node in graph.node_weights_mut() {
            let output_handles = node.get_all_output_handles().await;
            for output_handle in output_handles {
                let output_handle_id = output_handle.output_handle_id.clone();
                // 增加节点的出口连接数
                node.add_output_handle_connect_count(output_handle_id).await;
            }
        }
        // 再将所有的输出句柄添加到策略中
        for node in graph.node_weights_mut() {
            let output_handles = node.get_all_output_handles().await;
            strategy_output_handles.extend(output_handles);
        }
        strategy_output_handles
    }
}
