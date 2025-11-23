mod buid_position_node;
mod build_edge;
mod build_futures_order_node;
mod build_ifelse_node;
mod build_indicator_node;
mod build_kline_node;
mod build_leaf_node;
mod build_start_node;
mod build_variable_node;
mod symbol_config_checker;

// std
use std::str::FromStr;

// third-party
use snafu::OptionExt;
use strategy_core::{
    error::strategy_error::{EdgeConfigNullSnafu, NodeConfigNullSnafu},
    node::{
        NodeTrait, NodeType,
        context_trait::{NodeHandleExt, NodeInfoExt},
        node_trait::NodeContextAccessor,
    },
    strategy::context_trait::{StrategyBenchmarkExt, StrategyCommunicationExt, StrategyIdentityExt, StrategyInfraExt, StrategyWorkflowExt},
};
use tokio::sync::mpsc;
use virtual_trading::vts_trait::VtsCtxAccessor;

// workspace crate

// current crate
use super::BacktestStrategyContext;
use crate::{
    node::node_command::BacktestNodeCommand,
    strategy::strategy_error::{BacktestStrategyError, MissingDataSourceSnafu, MissingStartNodeSnafu},
};

impl BacktestStrategyContext {
    pub async fn build_workflow(&mut self) -> Result<(), BacktestStrategyError> {
        // get strategy config and clone to avoid borrow issues
        tracing::info!("[{}] start to build workflow", self.strategy_name());
        let node_config_list = self
            .strategy_config()
            .nodes
            .as_ref()
            .and_then(|node| node.as_array())
            .context(NodeConfigNullSnafu {
                strategy_name: self.strategy_name().clone(),
            })?
            .clone(); // 克隆以释放对 self 的借用

        // check node count, if only have one node(start node), throw error
        if node_config_list.len() == 1 {
            let node_config = node_config_list[0].clone();
            let node_type_str = node_config["type"].as_str().unwrap();
            let node_type = NodeType::from_str(node_type_str).unwrap();
            match node_type {
                NodeType::StartNode => {
                    let error = MissingDataSourceSnafu {
                        strategy_name: self.strategy_name().clone(),
                    }
                    .build();
                    return Err(error);
                }

                _ => {
                    let error = MissingStartNodeSnafu {
                        strategy_name: self.strategy_name().clone(),
                    }
                    .build();
                    return Err(error);
                }
            }
        }

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

                    let (node_id, selected_symbol_keys) = kline_node
                        .with_ctx_read(|ctx| (ctx.node_id().to_string(), ctx.selected_symbol_keys().clone()))
                        .await;

                    // set strategy keys
                    for (key, _) in selected_symbol_keys.iter() {
                        self.add_key(key.clone().into(), node_id.clone()).await;
                    }

                    self.add_node_command_sender(node_id, node_command_tx);

                    // add kline node event receiver to vitual_trading_system
                    let kline_node_event_receiver = kline_node
                        .with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle("virtual_trading_system".to_string()))
                        .await;
                    {
                        let virtual_trading_system = self.virtual_trading_system().lock().await;
                        virtual_trading_system
                            .with_ctx_write(|ctx| ctx.add_kline_node_event_receiver(kline_node_event_receiver))
                            .await;
                    }
                    self.add_node(kline_node.into()).await;
                }
                NodeType::IndicatorNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let indicator_node = self.build_indicator_node(node_config.clone(), node_command_rx).await?;
                    // set output handles
                    indicator_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;

                    let (node_id, indicator_keys) = indicator_node
                        .with_ctx_read(|ctx| (ctx.node_id().to_string(), ctx.indicator_keys().clone()))
                        .await;

                    // set strategy keys
                    for (key, _) in indicator_keys.iter() {
                        self.add_key(key.clone().into(), node_id.clone()).await;
                    }

                    self.add_node_command_sender(node_id, node_command_tx);
                    self.add_node(indicator_node.into()).await;
                }
                NodeType::IfElseNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let ifelse_node = self.build_ifelse_node(node_config.clone(), node_command_rx).await?;
                    // set output handles
                    ifelse_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;

                    let node_id = ifelse_node.with_ctx_read(|ctx| ctx.node_id().to_string()).await;
                    self.add_node_command_sender(node_id.clone(), node_command_tx);
                    self.add_node(ifelse_node.into()).await;
                }

                NodeType::FuturesOrderNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let (vts_command_sender, vts_event_receiver) = self
                        .virtual_trading_system()
                        .lock()
                        .await
                        .with_ctx_read(|ctx| (ctx.get_command_sender().clone(), ctx.get_vts_event_receiver().resubscribe()))
                        .await;
                    let futures_order_node = self
                        .build_futures_order_node(
                            node_config.clone(),
                            node_command_rx,
                            self.database().clone(),
                            self.heartbeat().clone(),
                            vts_command_sender,
                            vts_event_receiver,
                        )
                        .await?;
                    // set output handles
                    futures_order_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;

                    let node_id = futures_order_node.with_ctx_read(|ctx| ctx.node_id().to_string()).await;
                    self.add_node_command_sender(node_id.clone(), node_command_tx);
                    self.add_node(futures_order_node.into()).await;
                }
                NodeType::PositionNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let position_node = self
                        .build_position_node(
                            node_config.clone(),
                            node_command_rx,
                            self.database().clone(),
                            self.heartbeat().clone(),
                            self.virtual_trading_system().clone(),
                        )
                        .await?;
                    let node_id = position_node.with_ctx_read(|ctx| ctx.node_id().to_string()).await;
                    // set output handles
                    position_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;
                    self.add_node_command_sender(node_id, node_command_tx);
                    self.add_node(position_node.into()).await;
                }
                NodeType::VariableNode => {
                    let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);
                    let variable_node = self
                        .build_variable_node(node_config.clone(), node_command_rx, self.virtual_trading_system().clone())
                        .await?;
                    // set output handles
                    variable_node.with_ctx_write(|ctx| ctx.set_output_handles()).await;
                    let node_id = variable_node.with_ctx_read(|ctx| ctx.node_id().to_string()).await;
                    self.add_node_command_sender(node_id, node_command_tx);
                    self.add_node(variable_node.into()).await;
                }
            }
        }

        // build edges

        tracing::debug!("workflow build phase 2: build edges");
        let edge_config_list = self
            .strategy_config()
            .edges
            .as_ref()
            .and_then(|edge| edge.as_array())
            .context(EdgeConfigNullSnafu {
                strategy_name: self.strategy_name().clone(),
            })?
            .clone();

        for edge_config in edge_config_list {
            self.build_edge(edge_config).await?;
        }

        // check symbol config
        tracing::debug!("workflow build phase 3: check symbol config");
        if let Err(e) = self.check_symbol_config().await {
            return Err(e);
        }

        // set leaf nodes
        tracing::debug!("workflow build phase 4: set leaf nodes");
        self.build_leaf_nodes().await?;
        tracing::debug!("leaf node execution tracker: {:#?}", self.leaf_node_execution_tracker());

        tracing::debug!("workflow build phase 5: add node benchmark");
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
