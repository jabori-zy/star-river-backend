use std::sync::Arc;

use snafu::{IntoError, ResultExt};
use star_river_core::{
    custom_type::{NodeId, NodeName, StrategyId},
    error::StarRiverErrorTrait,
};
use strategy_core::{
    NodeType,
    benchmark::node_benchmark::CompletedCycle,
    error::node_error::{NodeError, StrategyCmdRespRecvFailedSnafu, StrategyCommandSendFailedSnafu},
    event::{log_event::NodeStateLogEvent, node_common_event::CommonEvent},
    node::{
        context_trait::{NodeInfoExt, NodeStateMachineExt},
        node_handles::NodeOutputHandle,
        node_state_machine::StateAction,
        node_trait::{NodeContextAccessor, NodeLifecycle},
    },
};
use tokio::{sync::mpsc, time::Duration};

use super::node_state_machine::{NodeRunState, NodeStateTransTrigger};
use crate::{
    node::node_event::BacktestNodeEvent,
    strategy::strategy_command::{AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand, BacktestStrategyCommand},
};

// current crate

pub struct NodeUtils;

impl NodeUtils {
    /// Helper function for mounting node cycle tracker
    ///
    /// # Core Logic
    /// 1. Create oneshot channel for receiving response
    /// 2. Build AddNodeCycleTrackerCommand
    /// 3. Send command through strategy_command_sender
    /// 4. Wait for and receive response
    ///
    /// # Parameters
    /// - `node_id`: Node ID
    /// - `cycle_tracker`: Completed cycle data
    /// - `strategy_command_sender`: Strategy command sender
    ///
    /// # Returns
    /// - `Ok(())`: Successfully mounted cycle tracker
    /// - `Err`: Failed to send command or receive response
    pub async fn mount_node_cycle_tracker(
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
        strategy_command_sender: &mpsc::Sender<BacktestStrategyCommand>,
    ) -> Result<(), NodeError> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let payload = AddNodeCycleTrackerCmdPayload::new(node_id.clone(), cycle_tracker);
        let command = AddNodeCycleTrackerCommand::new(node_id.clone(), resp_tx, payload).into();

        strategy_command_sender.send(command).await.map_err(|e| {
            StrategyCommandSendFailedSnafu {
                node_name: node_name.clone(),
            }
            .into_error(Arc::new(e))
        })?;

        resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: node_name.clone(),
        })?;

        Ok(())
    }

    pub async fn send_run_state_info(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        node_type: NodeType,
        msg: String,
        state: NodeRunState,
        action: impl StateAction,
        strategy_output_handle: &NodeOutputHandle<BacktestNodeEvent>,
    ) {
        let log_event: CommonEvent = NodeStateLogEvent::info(
            strategy_id,
            node_id,
            node_name,
            node_type.to_string(),
            state.to_string(),
            action.to_string(),
            msg,
        )
        .into();
        let _ = strategy_output_handle.send(log_event.into());
    }

    pub async fn send_run_state_error(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        node_type: NodeType,
        action: impl StateAction,
        error: &impl StarRiverErrorTrait,
        strategy_output_handle: &NodeOutputHandle<BacktestNodeEvent>,
    ) {
        let log_event: CommonEvent = NodeStateLogEvent::error(
            strategy_id,
            node_id,
            node_name,
            node_type.to_string(),
            NodeRunState::Failed.to_string(),
            action.to_string(),
            error,
        )
        .into();
        let _ = strategy_output_handle.send(log_event.into());
    }

    /// 通用节点初始化函数
    ///
    /// # 核心逻辑
    /// 1. 获取节点名称
    /// 2. 打印初始化开始日志
    /// 3. 触发状态转换: Created -> Initializing
    /// 4. (可选) 休眠指定时间
    /// 5. 获取当前状态
    /// 6. 打印初始化完成日志
    /// 7. 触发状态转换: Initializing -> Ready
    ///
    /// # 参数
    /// - `node`: 实现了 NodeLifecycle trait 的节点
    /// - `sleep_duration`: 可选的休眠时长(毫秒)
    ///
    /// # 返回
    /// - `Ok(())`: 初始化成功
    /// - `Err`: 状态转换失败
    pub async fn init_node<N>(node: &N, sleep_duration: Option<u64>) -> Result<(), N::Error>
    where
        N: NodeLifecycle<Trigger = NodeStateTransTrigger> + NodeContextAccessor,
        N::Context: NodeInfoExt + NodeStateMachineExt,
    {
        let node_name = node.with_ctx_read(|ctx| ctx.node_name().to_string()).await;
        tracing::info!("================={}====================", node_name);
        tracing::info!("[{node_name}] start init");

        // 开始初始化 Created -> Initializing
        node.update_node_state(NodeStateTransTrigger::StartInit).await?;

        // 可选的休眠时间
        if let Some(millis) = sleep_duration {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }

        let current_state = node
            .with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await.clone() }))
            .await;

        tracing::info!("[{node_name}] init complete: {:?}", current_state);

        // 初始化完成 Initializing -> Ready
        node.update_node_state(NodeStateTransTrigger::FinishInit).await?;

        Ok(())
    }

    /// 通用节点停止函数
    ///
    /// # 核心逻辑
    /// 1. 获取节点名称
    /// 2. 打印停止开始日志
    /// 3. 触发状态转换: Ready/Running -> Stopping
    /// 4. (可选) 休眠指定时间
    /// 5. 触发状态转换: Stopping -> Stopped
    ///
    /// # 参数
    /// - `node`: 实现了 NodeLifecycle trait 的节点
    /// - `sleep_duration`: 可选的休眠时长(毫秒)
    ///
    /// # 返回
    /// - `Ok(())`: 停止成功
    /// - `Err`: 状态转换失败
    pub async fn stop_node<N>(node: &N, sleep_duration: Option<u64>) -> Result<(), N::Error>
    where
        N: NodeLifecycle<Trigger = NodeStateTransTrigger> + NodeContextAccessor,
        N::Context: NodeInfoExt,
    {
        let node_name = node.with_ctx_read(|ctx| ctx.node_name().to_string()).await;
        tracing::info!("[{node_name}] start stop");

        // 开始停止 Ready/Running -> Stopping
        node.update_node_state(NodeStateTransTrigger::StartStop).await?;

        // 可选的休眠时间
        if let Some(millis) = sleep_duration {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }

        // 切换为stopped状态 Stopping -> Stopped
        node.update_node_state(NodeStateTransTrigger::FinishStop).await?;

        Ok(())
    }
}
