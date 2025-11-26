mod context;
mod kline_node_type;
mod node_lifecycle;
mod state_machine;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use context::KlineNodeContext;
use kline_node_type::KlineNodeBacktestConfig;
use snafu::{OptionExt, ResultExt};
use star_river_core::{
    custom_type::{NodeId, NodeName, StrategyId},
    state_machine::Metadata,
};
use strategy_core::{
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{NodeBase, NodeType, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle},
    strategy::cycle::Cycle,
};
use tokio::sync::{Mutex, RwLock, mpsc, watch};

use crate::{
    node::{
        node_command::BacktestNodeCommand,
        node_error::{
            KlineNodeError,
            kline_node_error::{ExchangeModeNotConfiguredSnafu, SymbolsIsNotConfiguredSnafuSnafu},
        },
        node_event::BacktestNodeEvent,
        node_state_machine::NodeRunState,
    },
    node_catalog::kline_node::state_machine::{KlineNodeStateMachine, kline_node_transition},
    strategy::strategy_command::BacktestStrategyCommand,
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
        cycle_rx: watch::Receiver<Cycle>,
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
    ) -> Result<Self, KlineNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_kline_node_config(node_config)?;

        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id, &node_name);

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
            cycle_rx,
            strategy_time_watch_rx,
            strategy_id,
            node_id,
            node_name,
            NodeType::KlineNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );

        let context = KlineNodeContext::new(metadata, node_config)?;
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_kline_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, KlineNodeBacktestConfig), KlineNodeError> {
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

        let mut kline_node_backtest_config = node_data
            .get("backtestConfig")
            .context(ConfigFieldValueNullSnafu {
                field_name: "backtestConfig".to_string(),
            })?
            .to_owned();

        // check data source account is configured

        kline_node_backtest_config
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedAccount"))
            .context(ExchangeModeNotConfiguredSnafu {
                node_name: node_name.clone(),
            })?;

        kline_node_backtest_config
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedSymbols"))
            .and_then(|symbols| symbols.as_array())
            .and_then(|arr| if arr.is_empty() { None } else { Some(arr) })
            .context(SymbolsIsNotConfiguredSnafuSnafu {
                node_name: node_name.clone(),
            })?;

        // Add nodeName to the backtest config
        if let Some(obj) = kline_node_backtest_config.as_object_mut() {
            obj.insert("nodeName".to_string(), serde_json::Value::String(node_name.clone()));
        }

        let node_config =
            serde_json::from_value::<KlineNodeBacktestConfig>(kline_node_backtest_config).context(ConfigDeserializationFailedSnafu {
                node_name: node_name.clone(),
            })?;

        Ok((strategy_id, node_id, node_name, node_config))
    }
}
