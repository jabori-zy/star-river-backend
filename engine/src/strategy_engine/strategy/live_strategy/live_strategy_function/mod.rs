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
use crate::strategy_engine::node::node_types::NodeMessageReceiver;
use petgraph::{Graph, Directed};


pub struct LiveStrategyFunction;


impl LiveStrategyFunction {
    pub async fn add_node_message_receivers(graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>) -> Vec<NodeMessageReceiver> {
        let mut strategy_message_receivers = Vec::new();
        for node in graph.node_weights_mut() {
            let message_receivers = node.get_message_receivers().await;
            strategy_message_receivers.extend(message_receivers);
        }
        strategy_message_receivers
    }
}
