mod state_machine;
mod context;
mod variable_node_type;
mod node_lifecycle;



use strategy_core::node::{NodeType, NodeBase};
use context::VariableNodeContext;
use strategy_core::node::node_trait::NodeContextAccessor;
use std::sync::Arc;
use tokio::sync::RwLock;
use star_river_core::custom_type::{StrategyId, NodeId, NodeName};
use crate::node::node_error::BacktestNodeError;
use crate::strategy::strategy_command::BacktestStrategyCommand;
use crate::node::node_command::BacktestNodeCommand;
use crate::strategy::PlayIndex;
use snafu::ResultExt;
use serde_json;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use variable_node_type::VariableNodeBacktestConfig;
use strategy_core::error::node_error::{ConfigFieldValueNullSnafu, ConfigDeserializationFailedSnafu};
use virtual_trading::VirtualTradingSystem;
use strategy_core::node::utils::generate_strategy_output_handle;
use state_machine::{variable_node_transition, VariableNodeStateMachine};
use strategy_core::node::metadata::NodeMetadata;
use crate::node::node_state_machine::NodeRunState;


#[derive(Debug, Clone)]
pub struct VariableNode {
    inner: NodeBase<VariableNodeContext>
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
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_variable_node_config(node_config)?;
        let strategy_output_handle = generate_strategy_output_handle(&node_id);
        let state_machine = VariableNodeStateMachine::new(
            node_name.clone(),
            NodeRunState::Created,
            variable_node_transition,
        );
        let metadata = NodeMetadata::new(
            strategy_id,
            node_id,
            node_name,
            NodeType::VariableNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = VariableNodeContext::new(metadata, node_config, play_index_watch_rx, virtual_trading_system);
        Ok(Self {
            inner: NodeBase::new(context)
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
            serde_json::from_value::<VariableNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}