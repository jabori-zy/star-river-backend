mod event_handler;
mod node_handles;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use snafu::ResultExt;
use star_river_core::custom_type::{NodeId, NodeName};
use star_river_event::backtest_strategy::node_event::{
    StartNodeEvent,
    start_node_event::{KlinePlayEvent, KlinePlayPayload},
};
use strategy_core::{
    benchmark::node_benchmark::{CompletedCycle, CycleTracker},
    error::node_error::StrategyCmdRespRecvFailedSnafu,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeHandleExt, NodeInfoExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};
use tokio::sync::RwLock;

use super::state_machine::StartNodeStateMachine;
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::StartNodeError, node_event::BacktestNodeEvent, node_utils::NodeUtils},
    strategy::{
        PlayIndex,
        strategy_command::{BacktestStrategyCommand, InitCustomVarCmdPayload, InitCustomVarValueCommand},
        strategy_config::BacktestStrategyConfig,
    },
};

pub type StartNodeMetadata = NodeMetadata<StartNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct StartNodeContext {
    pub metadata: StartNodeMetadata,
    pub node_config: Arc<RwLock<BacktestStrategyConfig>>,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl StartNodeContext {
    pub fn new(
        metadata: StartNodeMetadata,
        node_config: Arc<RwLock<BacktestStrategyConfig>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            play_index_watch_rx,
        }
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }

    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub async fn send_play_signal(&self) -> Result<(), StartNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
        cycle_tracker.start_phase("send_play_signal");
        let payload = KlinePlayPayload::new(self.play_index());
        let kline_play_event: StartNodeEvent = KlinePlayEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
            self.default_output_handle()?.output_handle_id().clone(),
            payload,
        )
        .into();
        self.default_output_handle_send(kline_play_event.into())?;
        cycle_tracker.end_phase("send_play_signal");
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    pub async fn init_custom_variables(&self) -> Result<(), StartNodeError> {
        let custom_var_configs = {
            let node_config_guard = self.node_config.read().await;
            node_config_guard.custom_variables.clone()
        };
        let (resp_rx, resp_tx) = tokio::sync::oneshot::channel();
        let init_var_payload = InitCustomVarCmdPayload::new(custom_var_configs);
        let init_var_cmd = InitCustomVarValueCommand::new(self.node_id().clone(), resp_rx, init_var_payload);
        self.send_strategy_command(init_var_cmd.into()).await?;
        let response = resp_tx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        if response.is_success() {
            return Ok(());
        }
        Ok(())
    }

    pub async fn current_time(&self) -> Result<DateTime<Utc>, StartNodeError> {
        NodeUtils::get_current_time_from_strategy(self.node_id().clone(), self.node_name().clone(), self.strategy_command_sender())
            .await
            .map_err(StartNodeError::from)
    }
}

impl NodeMetaDataExt for StartNodeContext {
    type StateMachine = StartNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for StartNodeContext {
    type Error = StartNodeError;

    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error> {
        crate::node::node_utils::NodeUtils::mount_node_cycle_tracker(node_id, node_name, cycle_tracker, self.strategy_command_sender())
            .await?;
        Ok(())
    }
}
