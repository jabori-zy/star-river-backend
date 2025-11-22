use star_river_core::custom_type::NodeId;
use tokio::sync::broadcast;

use super::node_handles::{HandleId, NodeOutputHandle};

pub fn generate_strategy_output_handle<E: Clone>(node_id: &NodeId) -> NodeOutputHandle<E> {
    let (tx, _) = broadcast::channel::<E>(100);
    let strategy_output_handle_id = format!("{}_strategy_output", node_id);
    let strategy_output_handle = NodeOutputHandle::new(node_id.clone(), false, -1, strategy_output_handle_id, tx);
    strategy_output_handle
}

pub fn generate_default_output_handle<E: Clone>(node_id: &NodeId) -> NodeOutputHandle<E> {
    let (tx, _) = broadcast::channel::<E>(100);
    let default_output_handle_id = generate_default_output_handle_id(node_id);
    let default_output_handle = NodeOutputHandle::new(node_id.clone(), true, -2, default_output_handle_id, tx);
    default_output_handle
}

pub fn generate_default_output_handle_id(node_id: &NodeId) -> HandleId {
    format!("{}_default_output", node_id)
}
