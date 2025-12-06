// third-party
use snafu::OptionExt;
use strategy_core::{
    error::strategy_error::{EdgeConfigMissFieldSnafu, NodeNotFoundByIdSnafu},
    node::node_handles::NodeInputHandle,
    strategy::context_trait::{StrategyIdentityExt, StrategyWorkflowExt},
};

// current crate
use super::BacktestStrategyContext;
use crate::strategy::strategy_error::BacktestStrategyError;

impl BacktestStrategyContext {
    pub async fn build_edge(&mut self, edge_config: serde_json::Value) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.strategy_name();

        let source_handle_id = edge_config.get("sourceHandle").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_name: strategy_name.clone(),
                field_name: "sourceHandle".to_string(),
            }
            .build()
        })?;

        let source_node_id = edge_config.get("source").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_name: strategy_name.clone(),
                field_name: "source".to_string(),
            }
            .build()
        })?;

        let target_node_id = edge_config.get("target").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_name: strategy_name.clone(),
                field_name: "target".to_string(),
            }
            .build()
        })?;

        let target_handle_id = edge_config.get("targetHandle").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_name: strategy_name.clone(),
                field_name: "targetHandle".to_string(),
            }
            .build()
        })?;

        let (source_idx, target_idx) = {
            let source = self.node_indices().get(source_node_id).copied().context(NodeNotFoundByIdSnafu {
                strategy_name: strategy_name.clone(),
                node_id: source_node_id.to_string(),
            })?;

            let target = self.node_indices().get(target_node_id).copied().context(NodeNotFoundByIdSnafu {
                strategy_name: strategy_name.clone(),
                node_id: target_node_id.to_string(),
            })?;

            (source, target)
        };

        tracing::debug!(
            "add edge: {:?} -> {:?}, source handle = {}",
            source_node_id,
            target_node_id,
            source_handle_id
        );

        // First get the output_handle of the source node
        let (_, receiver) = self
            .node(source_idx)?
            .subscribe_output_handle(source_handle_id.to_string(), target_handle_id.to_string())
            .await?;

        let target_node = self.node(target_idx)?;
        // Get receiver count
        let input_handle = NodeInputHandle::new(
            source_node_id.to_string(),
            source_handle_id.to_string(),
            target_handle_id.to_string(),
            receiver,
        );
        target_node.add_input_handle(input_handle).await;
        target_node.add_source_node(source_node_id.to_string()).await;

        // tracing::debug!("add edge: {:?} -> {:?}", from_node_id, to_node_id);
        self.graph_mut().add_edge(source_idx, target_idx, ());

        Ok(())
    }
}
