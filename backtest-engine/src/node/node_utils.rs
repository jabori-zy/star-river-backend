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
    event::{log_event::NodeRunStateLogEvent, node_common_event::CommonEvent},
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
        let log_event: CommonEvent = NodeRunStateLogEvent::info(
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
        let log_event: CommonEvent = NodeRunStateLogEvent::error(
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

    /// Generic node initialization function
    ///
    /// # Core Logic
    /// 1. Get node name
    /// 2. Print initialization start log
    /// 3. Trigger state transition: Created -> Initializing
    /// 4. (Optional) Sleep for specified duration
    /// 5. Get current state
    /// 6. Print initialization complete log
    /// 7. Trigger state transition: Initializing -> Ready
    ///
    /// # Parameters
    /// - `node`: Node implementing NodeLifecycle trait
    /// - `sleep_duration`: Optional sleep duration in milliseconds
    ///
    /// # Returns
    /// - `Ok(())`: Initialization successful
    /// - `Err`: State transition failed
    pub async fn init_node<N>(node: &N, sleep_duration: Option<u64>) -> Result<(), N::Error>
    where
        N: NodeLifecycle<Trigger = NodeStateTransTrigger> + NodeContextAccessor,
        N::Context: NodeInfoExt + NodeStateMachineExt,
    {
        let node_name = node.with_ctx_read(|ctx| ctx.node_name().to_string()).await;
        tracing::info!("================={}====================", node_name);
        tracing::info!("[{node_name}] start init");

        // Start initialization Created -> Initializing
        node.update_node_state(NodeStateTransTrigger::StartInit).await?;

        // Optional sleep duration
        if let Some(millis) = sleep_duration {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }

        let current_state = node
            .with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await.clone() }))
            .await;

        tracing::info!("[{node_name}] init complete: {:?}", current_state);

        // Initialization complete Initializing -> Ready
        node.update_node_state(NodeStateTransTrigger::FinishInit).await?;

        Ok(())
    }

    /// Generic node stop function
    ///
    /// # Core Logic
    /// 1. Get node name
    /// 2. Print stop start log
    /// 3. Trigger state transition: Ready/Running -> Stopping
    /// 4. (Optional) Sleep for specified duration
    /// 5. Trigger state transition: Stopping -> Stopped
    ///
    /// # Parameters
    /// - `node`: Node implementing NodeLifecycle trait
    /// - `sleep_duration`: Optional sleep duration in milliseconds
    ///
    /// # Returns
    /// - `Ok(())`: Stop successful
    /// - `Err`: State transition failed
    pub async fn stop_node<N>(node: &N, sleep_duration: Option<u64>) -> Result<(), N::Error>
    where
        N: NodeLifecycle<Trigger = NodeStateTransTrigger> + NodeContextAccessor,
        N::Context: NodeInfoExt,
    {
        let node_name = node.with_ctx_read(|ctx| ctx.node_name().to_string()).await;
        tracing::info!("@[{node_name}] start stop");

        // Start stopping Ready/Running -> Stopping
        node.update_node_state(NodeStateTransTrigger::StartStop).await?;

        // Optional sleep duration
        if let Some(millis) = sleep_duration {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }

        // Switch to stopped state Stopping -> Stopped
        node.update_node_state(NodeStateTransTrigger::FinishStop).await?;

        Ok(())
    }
}
