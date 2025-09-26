use super::{BacktestStrategyContext, BacktestStrategyFunction, NodeInputHandle};
use star_river_core::error::engine_error::strategy_error::*;
use std::sync::Arc;
use tokio::sync::RwLock;

impl BacktestStrategyFunction {
    pub async fn add_edge(
        // graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        // node_indices: &mut HashMap<String, NodeIndex>,
        context: Arc<RwLock<BacktestStrategyContext>>,
        edge_config: serde_json::Value,
        // from_node_id: &str,
        // from_handle_id: &str,
        // to_node_id: &str,
        // to_handle_id: &str
    ) -> Result<(), BacktestStrategyError> {
        let mut context_guard = context.write().await;

        let from_handle_id = edge_config.get("sourceHandle").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
                field_name: "sourceHandle".to_string(),
            }
            .build()
        })?;

        let from_node_id = edge_config.get("source").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
                field_name: "source".to_string(),
            }
            .build()
        })?;

        let to_node_id = edge_config.get("target").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
                field_name: "target".to_string(),
            }
            .build()
        })?;

        let to_handle_id = edge_config.get("targetHandle").and_then(|v| v.as_str()).ok_or_else(|| {
            EdgeConfigMissFieldSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
                field_name: "targetHandle".to_string(),
            }
            .build()
        })?;

        let (source, target) = {
            let source = context_guard.node_indices.get(from_node_id).copied().ok_or_else(|| {
                NodeNotFoundSnafu {
                    strategy_id: context_guard.strategy_id,
                    strategy_name: context_guard.strategy_name.clone(),
                    node_id: from_node_id.to_string(),
                }
                .build()
            })?;

            let target = context_guard.node_indices.get(to_node_id).copied().ok_or_else(|| {
                NodeNotFoundSnafu {
                    strategy_id: context_guard.strategy_id,
                    strategy_name: context_guard.strategy_name.clone(),
                    node_id: to_node_id.to_string(),
                }
                .build()
            })?;

            (source, target)
        };

        tracing::debug!(
            "add edge: {:?} -> {:?}, source handle = {}",
            from_node_id,
            to_node_id,
            from_handle_id
        );
        let from_node_handles = context_guard.graph.node_weight(source).unwrap().get_all_output_handles().await;
        tracing::debug!(
            "from_node_handles: {:?}",
            from_node_handles
                .iter()
                .map(|handle| handle.output_handle_id.clone())
                .collect::<Vec<String>>()
        );
        // 先获取源节点的output_handle
        let from_node_output_handle = context_guard
            .graph
            .node_weight(source)
            .unwrap()
            .get_output_handle(&from_handle_id.to_string())
            .await;
        // 增加源节点的出口连接数
        context_guard
            .graph
            .node_weight_mut(source)
            .unwrap()
            .add_output_handle_connect_count(&from_handle_id.to_string())
            .await;
        // tracing::debug!("sender: {:?}", sender);

        if let Some(target_node) = context_guard.graph.node_weight_mut(target) {
            let receiver = from_node_output_handle.subscribe();
            // 获取接收者数量
            let node_message_receiver = NodeInputHandle::new(
                from_node_id.to_string(),
                from_handle_id.to_string(),
                to_handle_id.to_string(),
                receiver,
            );
            target_node.add_message_receiver(node_message_receiver).await;
            let message_receivers = target_node.get_node_event_receivers().await;
            tracing::debug!(
                "[{}] have added message receivers: {:?}",
                target_node.get_node_name().await,
                message_receivers
                    .iter()
                    .map(|handle| handle.from_handle_id.clone())
                    .collect::<Vec<String>>()
            );
            target_node.add_from_node_id(from_node_id.to_string()).await;
        }
        // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
        context_guard.graph.add_edge(source, target, ());

        Ok(())
    }
}
