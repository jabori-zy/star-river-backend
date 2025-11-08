mod build_edge;
mod build_start_node;
mod build_kline_node;
mod symbol_config_checker;
mod build_leaf_node;

// std
use std::str::FromStr;

// third-party
use snafu::OptionExt;
use tokio::sync::mpsc;

// workspace crate

// current crate
use super::BacktestStrategyContext;
use crate::{
    error::strategy_error::{BacktestStrategyError, EdgeConfigNullSnafu, NodeConfigNullSnafu},
    node::{
        NodeType,
        node_context_trait::{NodeHandleTrait, NodeIdentity},
        node_trait::NodeContextAccessor,
    },
};

impl BacktestStrategyContext {
    pub async fn build_workflow(&mut self) -> Result<(), BacktestStrategyError> {
        // get strategy config and clone to avoid borrow issues
        tracing::info!("[{}] start to build workflow", &self.strategy_name);
        let node_config_list = self
            .strategy_config
            .nodes
            .as_ref()
            .and_then(|node| node.as_array())
            .context(NodeConfigNullSnafu {
                strategy_name: self.strategy_name.clone(),
            })?
            .clone(); // 克隆以释放对 self 的借用

        tracing::debug!("workflow build phase 1: build nodes");
        for node_config in node_config_list {
            let node_type_str = node_config["type"].as_str().unwrap();
            let node_type = NodeType::from_str(node_type_str).unwrap();
            match node_type {
                NodeType::StartNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let start_node = self.build_start_node(node_config.clone(), node_command_rx).await?;
                    // set output handles
                    start_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;
                    
                    let node_id = start_node.with_ctx_read(|ctx| ctx.node_id().to_string()).await;
                    self.add_node_command_sender(node_id, node_command_tx);
                    self.add_node(start_node.into()).await;
                }
                NodeType::KlineNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let kline_node = self.build_kline_node(node_config.clone(), node_command_rx).await?;
                    // set output handles
                    kline_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;
                    
                    let (node_id, selected_symbol_keys) = kline_node.with_ctx_read(|ctx| (ctx.node_id().to_string(), ctx.selected_symbol_keys().clone())).await;
                    
                    // set strategy keys
                    for (key, _) in selected_symbol_keys.iter() {
                        self.add_key(key.clone().into(), node_id.clone()).await;
                    }
                    
                    self.add_node_command_sender(node_id, node_command_tx);
                    self.add_node(kline_node.into()).await;
                }
                _ => {}
            }
        }

        // build edges

        tracing::debug!("workflow build phase 2: build edges");
        let edge_config_list = self
            .strategy_config
            .edges
            .as_ref()
            .and_then(|edge| edge.as_array())
            .context(EdgeConfigNullSnafu {
                strategy_name: self.strategy_name.clone(),
            })?
            .clone();

        for edge_config in edge_config_list {
            self.build_edge(edge_config).await?;
        }

        // check symbol config
        tracing::debug!("workflow build phase 3: check symbol config");
        // if let Err(e) = self.check_symbol_config().await {
        //     return Err(e);
        // }


        // set leaf nodes
        tracing::debug!("workflow build phase 4: set leaf nodes");
        self.build_leaf_nodes().await;


        // add node benchmark
        for node in self.topological_sort()?.iter() {
            let node_id = node.node_id().await;
            let node_name = node.node_name().await;
            let node_type = node.node_type().await.to_string();
            self.add_node_benchmark(node_id, node_name, node_type).await;
        }

        Ok(())
    }
}
