mod context;
mod node_lifecycle;
mod state_machine;
mod variable_node_type;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use context::VariableNodeContext;
use serde_json;
use snafu::ResultExt;
use star_river_core::custom_type::{CycleId, NodeId, NodeName, StrategyId};
use state_machine::{VariableNodeStateMachine, variable_node_transition};
use strategy_core::{
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{NodeBase, NodeType, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle}, strategy::cycle::Cycle,
};
use tokio::sync::{Mutex, RwLock, mpsc, watch};
use variable_node_type::VariableNodeBacktestConfig;

use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError, node_state_machine::NodeRunState},
    strategy::strategy_command::BacktestStrategyCommand,
    virtual_trading_system::BacktestVts,
};

#[derive(Debug, Clone)]
pub struct VariableNode {
    inner: NodeBase<VariableNodeContext>,
}

impl std::ops::Deref for VariableNode {
    type Target = NodeBase<VariableNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for VariableNode {
    type Context = VariableNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.inner.context
    }
}

impl VariableNode {
    pub fn new(
        cycle_rx: watch::Receiver<Cycle>,
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        virtual_trading_system: Arc<Mutex<BacktestVts>>,
        current_time_watch_rx: tokio::sync::watch::Receiver<DateTime<Utc>>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_variable_node_config(node_config)?;
        let strategy_output_handle = generate_strategy_output_handle(&node_id);
        let state_machine = VariableNodeStateMachine::new(node_name.clone(), NodeRunState::Created, variable_node_transition);
        let metadata = NodeMetadata::new(
            cycle_rx,
            strategy_id,
            node_id,
            node_name,
            NodeType::VariableNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = VariableNodeContext::new(metadata, node_config, virtual_trading_system, current_time_watch_rx);
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_variable_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, VariableNodeBacktestConfig), BacktestNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "id".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "data".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "nodeName".to_string(),
                }
                .build()
            })?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "strategyId".to_string(),
                }
                .build()
            })?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let backtest_config =
            serde_json::from_value::<VariableNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {
                node_name: node_name.clone(),
            })?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}
