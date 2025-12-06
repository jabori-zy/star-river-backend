mod context;
mod evaluate;
mod if_else_node_type;
mod node_lifecycle;
mod state_machine;
mod utils;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use context::IfElseNodeContext;
use if_else_node_type::IfElseNodeBacktestConfig;
use snafu::ResultExt;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use state_machine::{IfElseNodeStateMachine, if_else_node_transition};
use strategy_core::{
    NodeType,
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{NodeBase, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle},
    node_infra::if_else_node::Case,
    strategy::cycle::Cycle,
};
use tokio::sync::{Mutex, RwLock, mpsc, watch};

use crate::{
    node::{
        node_command::BacktestNodeCommand, node_error::BacktestNodeError, node_event::BacktestNodeEvent, node_state_machine::NodeRunState,
    },
    strategy::strategy_command::BacktestStrategyCommand,
};

#[derive(Debug, Clone)]
pub struct IfElseNode {
    inner: NodeBase<IfElseNodeContext>,
}

impl std::ops::Deref for IfElseNode {
    type Target = NodeBase<IfElseNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for IfElseNode {
    type Context = IfElseNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}

impl IfElseNode {
    pub fn new(
        cycle_rx: watch::Receiver<Cycle>,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) = Self::check_if_else_node_config(node_config)?;

        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id, &node_name);

        let state_machine = IfElseNodeStateMachine::new(node_name.clone(), NodeRunState::Created, if_else_node_transition);
        let metadata = NodeMetadata::new(
            cycle_rx,
            strategy_time_watch_rx,
            strategy_id,
            node_id,
            node_name,
            NodeType::IfElseNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let is_nested = backtest_config.is_nested;
        let context = IfElseNodeContext::new(metadata, backtest_config, is_nested);
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_if_else_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, IfElseNodeBacktestConfig), BacktestNodeError> {
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

        let is_nested = node_data.get("isNested").and_then(|is_nested| is_nested.as_bool()).ok_or_else(|| {
            ConfigFieldValueNullSnafu {
                field_name: "isNested".to_string(),
            }
            .build()
        })?;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let cases_json = backtest_config_json
            .get("cases")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "cases".to_string(),
                }
                .build()
            })?
            .to_owned();
        let cases = serde_json::from_value::<Vec<Case>>(cases_json).context(ConfigDeserializationFailedSnafu {
            node_name: node_name.clone(),
        })?;
        let backtest_config = IfElseNodeBacktestConfig { cases, is_nested };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}
