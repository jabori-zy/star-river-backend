mod context;
mod node_lifecycle;
mod position_node_types;
mod state_machine;

use std::sync::Arc;

use context::PositionNodeContext;
use heartbeat::Heartbeat;
use position_node_types::PositionNodeBacktestConfig;
use sea_orm::DatabaseConnection;
use serde_json;
use snafu::ResultExt;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use state_machine::{PositionNodeStateMachine, position_node_transition};
use strategy_core::{
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{NodeBase, NodeType, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle},
};
use tokio::sync::{Mutex, RwLock, mpsc};

use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError, node_state_machine::NodeRunState},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
    virtual_trading_system::BacktestVts,
};

#[derive(Debug, Clone)]
pub struct PositionNode {
    inner: NodeBase<PositionNodeContext>,
}

impl std::ops::Deref for PositionNode {
    type Target = NodeBase<PositionNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for PositionNode {
    type Context = PositionNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.inner.context
    }
}

impl PositionNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        virtual_trading_system: Arc<Mutex<BacktestVts>>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_position_node_config(node_config)?;
        let strategy_output_handle = generate_strategy_output_handle(&node_id);
        let state_machine = PositionNodeStateMachine::new(node_name.clone(), NodeRunState::Created, position_node_transition);
        let metadata = NodeMetadata::new(
            strategy_id,
            node_id,
            node_name,
            NodeType::PositionNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = PositionNodeContext::new(
            metadata,
            node_config,
            play_index_watch_rx,
            database,
            heartbeat,
            virtual_trading_system,
        );
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_position_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, PositionNodeBacktestConfig), BacktestNodeError> {
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
            serde_json::from_value::<PositionNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}
