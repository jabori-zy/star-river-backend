// ============================================================================
// 子模块声明
// ============================================================================

pub mod context;
pub mod kline_node_type;
pub mod state_machine;
pub mod node_lifecycle;

// ============================================================================
// 重新导出（对外）
// ============================================================================

pub use context::KlineNodeContext;
pub use state_machine::{KlineNodeStateMachine, kline_node_transition, KlineNodeAction};
pub use kline_node_type::KlineNodeBacktestConfig;

// ============================================================================
// 标准库导入
// ============================================================================

use std::sync::Arc;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use event_center::communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender};
use snafu::{OptionExt, ResultExt};
use star_river_core::custom_type::{NodeId, NodeName, PlayIndex, StrategyId};
use crate::error::node_error::{
    BacktestNodeError, ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu,
};
use tokio::sync::{Mutex, RwLock};

// ============================================================================
// 当前模块内部导入
// ============================================================================

use crate::node::NodeType;
use crate::node::NodeBase;
use crate::node::base_context::NodeBaseContext;
use crate::node::node_state_machine::{NodeRunState, Metadata};
use crate::node::node_utils::NodeUtils;
use crate::node::node_trait::NodeContextAccessor;

// ============================================================================
// KlineNode 结构 (newtype 模式)
// ============================================================================

/// K线节点
#[derive(Debug, Clone)]
pub struct KlineNode {
    inner: NodeBase<KlineNodeContext, KlineNodeAction>,
}

impl std::ops::Deref for KlineNode {
    type Target = NodeBase<KlineNodeContext, KlineNodeAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor<KlineNodeContext, KlineNodeAction> for KlineNode {
    fn context(&self) -> &Arc<RwLock<KlineNodeContext>> {
        self.inner.context()
    }
}


impl KlineNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_kline_node_config(node_config)?;


        let metadata = match serde_json::to_string(&node_config.data_source) {
            Ok(json_str) => Metadata::from_json(&json_str).ok(),
            Err(_) => None,
        };

        let state_machine = KlineNodeStateMachine::with_metadata(
            node_name.clone(),
            NodeRunState::Created,
            kline_node_transition,
            metadata
        );
        let strategy_output_handle = NodeUtils::generate_strategy_output_handle(&node_id);
        let base_context = NodeBaseContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::KlineNode,
            state_machine,
            strategy_output_handle,
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );
        let context = KlineNodeContext::new(base_context, node_config);

        Ok(Self {
            inner: NodeBase::new(context)
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
            serde_json::from_value::<KlineNodeBacktestConfig>(kline_node_backtest_config)
            .context(ConfigDeserializationFailedSnafu {})?;

        Ok((strategy_id, node_id, node_name, node_config))
    }
}
