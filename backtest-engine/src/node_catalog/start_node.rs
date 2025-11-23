mod context;
mod handle_play_index;
mod node_lifecycle;
mod state_machine;

// ============================================================================
// 重新导出（对外）
// ============================================================================

// ============================================================================
// 标准库导入
// ============================================================================
use std::sync::Arc;

pub use context::StartNodeContext;
use snafu::{OptionExt, ResultExt};
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
pub use state_machine::{StartNodeStateMachine, start_node_transition};
use strategy_core::{
    error::node_error::{
        ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu, ValueNotGreaterThanOrEqualToZeroSnafu, ValueNotGreaterThanZeroSnafu,
    },
    node::{NodeBase, NodeType, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle},
};
// use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{Mutex, RwLock, mpsc};

// ============================================================================
// 外部 crate 导入
// ============================================================================
use crate::node::node_error::BacktestNodeError;
use crate::strategy::strategy_config::BacktestStrategyConfig;
// use virtual_trading::VirtualTradingSystem;

// ============================================================================
// 当前模块内部导入
// ============================================================================
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent, node_state_machine::NodeRunState},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

// ============================================================================
// StartNode 结构 (newtype 模式)
// ============================================================================

/// 起始节点
#[derive(Debug, Clone)]
pub struct StartNode {
    inner: NodeBase<StartNodeContext>,
}

impl std::ops::Deref for StartNode {
    type Target = NodeBase<StartNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for StartNode {
    type Context = StartNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}

impl StartNode {
    /// 创建新的 StartNode 实例
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, backtest_strategy_config) = Self::check_start_node_config(node_config)?;
        let strategy_output_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id);

        let state_machine = StartNodeStateMachine::new(node_name.clone(), NodeRunState::Created, start_node_transition);

        let metadata = NodeMetadata::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::StartNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
        );

        let context = StartNodeContext::new(metadata, Arc::new(RwLock::new(backtest_strategy_config)), play_index_watch_rx);

        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_start_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, BacktestStrategyConfig), BacktestNodeError> {
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
        let backtest_config_json = node_data
            .get("backtestConfig")
            .context(ConfigFieldValueNullSnafu {
                field_name: "backtestConfig".to_string(),
            })?
            .to_owned();

        let backtest_strategy_config =
            serde_json::from_value::<BacktestStrategyConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {
                node_name: node_name.clone(),
            })?;

        // check initial balance (> 0)
        if backtest_strategy_config.initial_balance <= 0.0 {
            let error = ValueNotGreaterThanZeroSnafu {
                node_name: node_name.clone(),
                config_name: "initial balance".to_string(),
                config_value: backtest_strategy_config.initial_balance,
            }
            .build()
            .into();
            return Err(error);
        }

        // check leverage (> 0)
        if backtest_strategy_config.leverage <= 0 {
            let error = ValueNotGreaterThanZeroSnafu {
                node_name: node_name.clone(),
                config_name: "leverage".to_string(),
                config_value: backtest_strategy_config.leverage as f64,
            }
            .build()
            .into();
            return Err(error);
        }

        // check fee rate (>= 0)
        if backtest_strategy_config.fee_rate < 0.0 {
            let error = ValueNotGreaterThanOrEqualToZeroSnafu {
                node_name: node_name.clone(),
                config_name: "fee rate".to_string(),
                config_value: backtest_strategy_config.fee_rate,
            }
            .build()
            .into();
            return Err(error);
        }

        Ok((strategy_id, node_id, node_name, backtest_strategy_config))
    }
}
