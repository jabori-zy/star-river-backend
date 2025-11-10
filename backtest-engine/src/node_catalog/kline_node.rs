mod context;
mod kline_node_type;
mod node_lifecycle;
mod state_machine;

use std::sync::Arc;

use context::KlineNodeContext;
use kline_node_type::KlineNodeBacktestConfig;
use snafu::{OptionExt, ResultExt};
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use strategy_core::{
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{
        NodeBase, NodeType, metadata::NodeMetadata, node_state_machine::Metadata, node_trait::NodeContextAccessor,
        utils::generate_strategy_output_handle,
    },
};
use tokio::sync::{Mutex, RwLock, mpsc};

use crate::{
    node::{
        node_command::BacktestNodeCommand, node_error::BacktestNodeError, node_event::BacktestNodeEvent, node_state_machine::NodeRunState,
    },
    node_catalog::kline_node::state_machine::{KlineNodeStateMachine, kline_node_transition},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

#[derive(Debug, Clone)]
pub struct KlineNode {
    inner: NodeBase<KlineNodeContext>,
}

impl std::ops::Deref for KlineNode {
    type Target = NodeBase<KlineNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for KlineNode {
    type Context = KlineNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}

impl KlineNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_kline_node_config(node_config)?;

        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id);

        let state_machine_metadata = match serde_json::to_string(&node_config.data_source) {
            Ok(json_str) => Metadata::from_json(&json_str).ok(),
            Err(_) => None,
        };

        let state_machine = KlineNodeStateMachine::with_metadata(
            node_name.clone(),
            NodeRunState::Created,
            kline_node_transition,
            state_machine_metadata,
        );

        let metadata = NodeMetadata::new(
            strategy_id,
            node_id,
            node_name,
            NodeType::KlineNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = KlineNodeContext::new(metadata, node_config, play_index_watch_rx);
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_kline_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, KlineNodeBacktestConfig), BacktestNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .context(ConfigFieldValueNullSnafu {
                field_name: "id".to_string(),
            })?
            .to_owned();

        let node_data = node_config
            .get("data")
            .context(ConfigFieldValueNullSnafu {
                field_name: "data".to_string(),
            })?
            .to_owned();

        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .context(ConfigFieldValueNullSnafu {
                field_name: "nodeName".to_string(),
            })?
            .to_owned();

        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .context(ConfigFieldValueNullSnafu {
                field_name: "strategyId".to_string(),
            })?
            .to_owned() as StrategyId;
        let kline_node_backtest_config = node_data
            .get("backtestConfig")
            .context(ConfigFieldValueNullSnafu {
                field_name: "backtestConfig".to_string(),
            })?
            .to_owned();

        let node_config =
            serde_json::from_value::<KlineNodeBacktestConfig>(kline_node_backtest_config).context(ConfigDeserializationFailedSnafu {})?;

        Ok((strategy_id, node_id, node_name, node_config))
    }
}
