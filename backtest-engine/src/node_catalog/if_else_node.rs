mod state_machine;
mod context;
mod if_else_node_type;
mod utils;
mod evaluate;
mod node_lifecycle;


use context::IfElseNodeContext;
use strategy_core::node::NodeBase;
use strategy_core::node::node_trait::NodeContextAccessor;
use std::sync::Arc;
use tokio::sync::RwLock;
use star_river_core::{
    custom_type::{NodeId, NodeName, StrategyId},
};
use strategy_core::{
    NodeType,
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{
        metadata::NodeMetadata,
        utils::generate_strategy_output_handle,
    },
};
use if_else_node_type::IfElseNodeBacktestConfig;
use crate::node::node_error::BacktestNodeError;
use tokio::sync::{Mutex, mpsc};
use crate::strategy::strategy_command::BacktestStrategyCommand;
use crate::node::node_command::BacktestNodeCommand;
use crate::strategy::PlayIndex;
use strategy_core::node_infra::if_else_node::Case;
use snafu::ResultExt;
use crate::node::node_event::BacktestNodeEvent;
use state_machine::{IfElseNodeStateMachine, if_else_node_transition};
use crate::node::node_state_machine::NodeRunState;

#[derive(Debug, Clone)]
pub struct IfElseNode {
    inner: NodeBase<IfElseNodeContext>
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
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, BacktestNodeError> {

        let (strategy_id, node_id, node_name, backtest_config) = Self::check_if_else_node_config(node_config)?;

        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id);

        let state_machine = IfElseNodeStateMachine::new(
            node_name.clone(),
            NodeRunState::Created,
            if_else_node_transition,
        );
        let metadata = NodeMetadata::new(
            strategy_id,
            node_id,
            node_name,
            NodeType::IfElseNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = IfElseNodeContext::new(
            metadata,
            backtest_config,
            play_index_watch_rx,
        );
        Ok(Self {
            inner: NodeBase::new(context)
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
        let cases = serde_json::from_value::<Vec<Case>>(cases_json).context(ConfigDeserializationFailedSnafu {})?;
        let backtest_config = IfElseNodeBacktestConfig { cases };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}