// third-party
use snafu::OptionExt;

// current crate
use super::BacktestStrategyContext;
use crate::{
    error::strategy_error::{BacktestStrategyError, EdgeConfigMissFieldSnafu, NodeNotFoundSnafu},
    node::node_handles::NodeInputHandle,
};

impl BacktestStrategyContext {
    pub async fn build_edge(&mut self, edge_config: serde_json::Value) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.strategy_name.clone();
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

        let (source, target) = {
            let source = self.node_indices.get(source_node_id).copied().context(NodeNotFoundSnafu {
                strategy_name: strategy_name.clone(),
                node_id: source_node_id.to_string(),
            })?;

            let target = self.node_indices.get(target_node_id).copied().context(NodeNotFoundSnafu {
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

        // 先获取源节点的output_handle
        let receiver = self
            .graph
            .node_weight(source)
            .unwrap()
            .subscribe_output_handle(source_handle_id.to_string(), target_handle_id.to_string())
            .await;

        if let Some(target_node) = self.graph.node_weight_mut(target) {
            // let receiver = from_node_output_handle.subscribe(target_handle_id.to_string());
            // 获取接收者数量
            let input_handle = NodeInputHandle::new(
                source_node_id.to_string(),
                source_handle_id.to_string(),
                target_handle_id.to_string(),
                receiver,
            );
            target_node.add_input_handle(input_handle).await;
            target_node.add_source_node(source_node_id.to_string()).await;
        }
        // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
        self.graph.add_edge(source, target, ());

        Ok(())
    }
}
