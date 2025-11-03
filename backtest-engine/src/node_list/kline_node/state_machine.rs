use crate::node::node_state_machine::{Metadata, NodeRunState, NodeStateMachine, NodeStateTransTrigger, StateChangeActions};
use crate::error::node_state_machine_error::{BacktestNodeStateMachineError, NodeTransFailedSnafu};
use star_river_core::strategy::BacktestDataSource;
use strum::Display;

/// KlineNode 状态机类型别名
pub type KlineNodeStateMachine = NodeStateMachine<KlineNodeAction>;

// KlineNode 状态转换后需要执行的动作
#[derive(Debug, Clone, Display)]
pub enum KlineNodeAction {
    #[strum(serialize = "ListenAndHandleExternalEvents")]
    ListenAndHandleExternalEvents, // 处理外部事件

    #[strum(serialize = "ListenAndHandleNodeEvents")]
    ListenAndHandleNodeEvents, // 监听节点消息

    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 处理策略命令

    #[strum(serialize = "LogNodeState")]
    LogNodeState, // 记录节点状态

    #[strum(serialize = "GetMinIntervalSymbols")]
    GetMinIntervalSymbols, // 获取最小周期交易对

    #[strum(serialize = "RegisterExchange")]
    RegisterExchange, // 注册交易所

    #[strum(serialize = "LoadHistoryFromExchange")]
    LoadHistoryFromExchange, // 从交易所加载K线历史

    #[strum(serialize = "LoadHistoryFromFile")]
    LoadHistoryFromFile, // 从文件加载K线历史

    #[strum(serialize = "LogTransition")]
    LogTransition, // 记录状态转换

    #[strum(serialize = "LogError")]
    LogError(String), // 记录错误

    #[strum(serialize = "CancelAsyncTask")]
    CancelAsyncTask, // 取消异步任务
}

/// KlineNode 状态转换函数
///
/// 根据 data_source 元数据决定不同的初始化流程
pub fn kline_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    metadata: Option<&Metadata>,
) -> Result<StateChangeActions<KlineNodeAction>, BacktestNodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => {
            // 从 metadata 中读取 data_source，决定不同的初始化动作
            let data_source = metadata
                .and_then(|m| m.get::<BacktestDataSource>("data_source"))
                .unwrap_or(BacktestDataSource::Exchange); // 默认使用文件加载

            let actions = match data_source {
                BacktestDataSource::Exchange => vec![
                    KlineNodeAction::LogTransition,
                    KlineNodeAction::LogNodeState,
                    KlineNodeAction::ListenAndHandleExternalEvents,
                    KlineNodeAction::ListenAndHandleNodeEvents,
                    KlineNodeAction::ListenAndHandleStrategyCommand,
                    KlineNodeAction::GetMinIntervalSymbols,
                    KlineNodeAction::RegisterExchange,
                    KlineNodeAction::LoadHistoryFromExchange,
                ],
                BacktestDataSource::File => vec![
                    KlineNodeAction::LogTransition,
                    KlineNodeAction::LogNodeState,
                    KlineNodeAction::ListenAndHandleExternalEvents,
                    KlineNodeAction::ListenAndHandleNodeEvents,
                    KlineNodeAction::ListenAndHandleStrategyCommand,
                    KlineNodeAction::LoadHistoryFromFile,
                ],
            };

            Ok(StateChangeActions::new(NodeRunState::Initializing, actions))
        }

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => {
            Ok(StateChangeActions::new(
                NodeRunState::Ready,
                vec![KlineNodeAction::LogTransition, KlineNodeAction::LogNodeState],
            ))
        }

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![
                KlineNodeAction::LogTransition,
                KlineNodeAction::LogNodeState,
                KlineNodeAction::CancelAsyncTask,
            ],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![KlineNodeAction::LogTransition, KlineNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![
                KlineNodeAction::LogTransition,
                KlineNodeAction::LogNodeState,
                KlineNodeAction::LogError(error.clone()),
            ],
        )),

        // Invalid transition
        _ => NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}

/// 创建 KlineNode 的 Metadata
///
/// # Arguments
/// * `data_source` - 数据源类型（File 或 Exchange）
pub fn create_kline_metadata(data_source: BacktestDataSource) -> Result<Metadata, serde_json::Error> {
    let mut metadata_map = std::collections::HashMap::new();
    metadata_map.insert("data_source".to_string(), serde_json::to_value(data_source)?);
    Ok(Metadata::from_map(metadata_map))
}
