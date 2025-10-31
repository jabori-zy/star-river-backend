// ============================================================================
// 子模块声明
// ============================================================================

pub mod context;
pub mod state_machine;
pub mod node_lifecycle;
mod handle_play_index;

// ============================================================================
// 重新导出（对外）
// ============================================================================

pub use context::StartNodeContext;
pub use state_machine::{StartNodeStateMachine, start_node_transition};

// ============================================================================
// 标准库导入
// ============================================================================

use std::sync::Arc;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use event_center::communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender};
use snafu::{OptionExt, ResultExt};
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use star_river_core::error::node_error::backtest_node_error::{
    BacktestNodeError, ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu,
    start_node_error::{ValueNotGreaterThanOrEqualToZeroSnafu, ValueNotGreaterThanZeroSnafu},
};
use star_river_core::strategy::BacktestStrategyConfig;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{Mutex, RwLock};
use virtual_trading::VirtualTradingSystem;

// ============================================================================
// 当前模块内部导入
// ============================================================================

use crate::backtest_engine::node::NodeType;
use crate::backtest_engine::node::StartNode;
use crate::backtest_engine::node::base_context::NodeBaseContext;
use crate::backtest_engine::node::node_state_machine::NodeRunState;
use crate::backtest_engine::node::node_utils::NodeUtils;

impl StartNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<i32>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, backtest_strategy_config) = Self::check_start_node_config(node_config)?;
        let strategy_output_handle = NodeUtils::generate_strategy_output_handle(&node_id);

        let state_machine = StartNodeStateMachine::new(node_name.clone(), NodeRunState::Created, start_node_transition);

        let base_context = NodeBaseContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::StartNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );
        let context = StartNodeContext::new(
            base_context,
            Arc::new(RwLock::new(backtest_strategy_config)),
            virtual_trading_system,
            strategy_stats,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(context)),
            _phantom: std::marker::PhantomData,
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
            serde_json::from_value::<BacktestStrategyConfig>(backtest_config_json)
            .context(ConfigDeserializationFailedSnafu {})?;

        // check initial balance (> 0)
        if backtest_strategy_config.initial_balance <= 0.0 {
            let error = ValueNotGreaterThanZeroSnafu {
                node_name: node_name.clone(),
                node_id: node_id.clone(),
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
                node_id: node_id.clone(),
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
                node_id: node_id.clone(),
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
