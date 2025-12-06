// std
use std::sync::Arc;

use chrono::{DateTime, Utc};
use event_center::EventCenterSingleton;
use snafu::ResultExt;
use star_river_core::custom_type::StrategyId;
use star_river_event::backtest_strategy::strategy_event::{BacktestStrategyEvent, PlayFinishedEvent};
use strategy_core::{
    benchmark::strategy_benchmark::StrategyCycleTracker,
    error::strategy_error::NodeCmdRespRecvFailedSnafu,
    node::NodeTrait,
    strategy::{
        context_trait::{
            StrategyBenchmarkExt, StrategyCommunicationExt, StrategyIdentityExt, StrategyInfoExt, StrategyVariableExt, StrategyWorkflowExt,
        },
        cycle::Cycle,
    },
};
use strategy_stats::strategy_stats::StrategyStatsAccessor;
// third-party
use tokio::sync::{Mutex, Notify, RwLock, oneshot, watch};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use virtual_trading::vts_trait::VtsCtxAccessor;

// current crate
use super::BacktestStrategyContext;
// workspace crate
use crate::{
    node::BacktestNode,
    strategy::{PlayIndex, signal_generator::SignalGenerator},
};
use crate::{
    node::node_command::{NodeResetCmdPayload, NodeResetCommand},
    strategy::{
        strategy_error::{AlreadyPausingSnafu, AlreadyPlayingSnafu, BacktestStrategyError, PlayFinishedSnafu},
        strategy_state_machine::BacktestStrategyRunState,
    },
};

#[derive(Debug)]
struct PlayContext {
    strategy_id: StrategyId,
    strategy_name: String,
    node: BacktestNode,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    child_cancel_play_token: CancellationToken,
    execute_over_notify: Arc<Notify>,
    cycle_tracker: Arc<RwLock<Option<StrategyCycleTracker>>>,
    signal_generator: Arc<Mutex<SignalGenerator>>,
    current_time_watch_tx: watch::Sender<DateTime<Utc>>,
    cycle_watch_tx: watch::Sender<Cycle>,
}

impl BacktestStrategyContext {
    // Check and set playing state
    async fn check_and_set_playing_state(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("{}: already playing, no need to play again", self.strategy_name());
            return false;
        }
        *self.is_playing.write().await = true;
        true
    }

    async fn create_play_context(&self) -> PlayContext {
        let node = self.get_node("start_node").unwrap();

        PlayContext {
            strategy_id: self.strategy_id(),
            strategy_name: self.strategy_name().clone(),
            node: node.clone(),
            is_playing: self.is_playing.clone(),
            initial_play_speed: self.initial_play_speed.clone(),
            child_cancel_play_token: self.cancel_play_token.child_token(),
            execute_over_notify: self.execute_over_notify.clone(),
            cycle_tracker: self.cycle_tracker().clone(),
            signal_generator: self.signal_generator.clone(),
            current_time_watch_tx: self.strategy_time_watch_tx().clone(),
            cycle_watch_tx: self.cycle_watch_tx().clone(),
        }
    }

    async fn run_play_loop(context: PlayContext) {
        loop {
            // Check cancellation status
            if context.child_cancel_play_token.is_cancelled() {
                tracing::info!("[{}]: received cancel signal, exit play task", context.strategy_name.clone());
                *context.is_playing.write().await = false;
                break;
            }

            // Check pause status
            if let Some(should_break) = Self::handle_pause_state(&context, &context.strategy_name).await {
                if should_break {
                    break;
                }
                continue;
            }

            // Get play speed
            let play_speed = Self::get_play_speed(&context).await;

            // Check if signal generator has finished
            let mut signal_generator_guard = context.signal_generator.lock().await;
            let is_finished = signal_generator_guard.is_finished();

            if is_finished {
                drop(signal_generator_guard);
                tracing::info!("[{}]: all signals played, exit play task", context.strategy_name);
                *context.is_playing.write().await = false;
                break;
            }

            // Get next signal
            let (signal_index, signal_time) = signal_generator_guard.next().unwrap();
            let is_finished_after_next = signal_generator_guard.is_finished();
            drop(signal_generator_guard);

            // tracing::debug!(
            //     "[{}]: playing signal_index: {}, signal_time: {}, progress: {}",
            //     context.strategy_name,
            //     signal_index,
            //     signal_time,
            //     progress
            // );

            // Start single cycle logic
            let mut strategy_cycle_tracker = StrategyCycleTracker::new(signal_index);
            strategy_cycle_tracker.start_phase("increment play index");

            context.current_time_watch_tx.send(signal_time).unwrap();
            context.cycle_watch_tx.send(Cycle::Id(signal_index)).unwrap();

            // Explicitly release cycle_tracker lock to avoid holding lock while waiting for notify
            strategy_cycle_tracker.end_phase("increment play index");
            {
                let mut cycle_tracker_guard = context.cycle_tracker.write().await;
                *cycle_tracker_guard = Some(strategy_cycle_tracker); // Share to strategy context
            }
            // After sending, wait for all leaf nodes to complete execution
            context.execute_over_notify.notified().await;

            // Check if playback finished
            if is_finished_after_next {
                Self::handle_play_finished(&context, &context.strategy_name, signal_index as i32).await;
                break;
            }

            // Playback delay
            // play_speed represents how many klines to play per second, 100 means 100 klines per second
            // 1000 / 100 = 10ms
            let delay_millis = 1000 / play_speed as u64;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)).await;
        }
    }

    // Handle pause state
    async fn handle_pause_state(context: &PlayContext, strategy_name: &str) -> Option<bool> {
        if !*context.is_playing.read().await {
            let signal_generator_guard = context.signal_generator.lock().await;
            let total_count = signal_generator_guard.total_signal_count();
            let current_index = signal_generator_guard.current_index();
            drop(signal_generator_guard);

            tracing::info!(
                "[{}]: pause play, total_signal_count: {}, current_index: {}",
                context.node.node_id().await,
                total_count,
                current_index
            );

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    return Some(false); // continue
                }
                _ = context.child_cancel_play_token.cancelled() => {
                    tracing::info!("[{}]: received cancel signal, exit play task", strategy_name);
                    *context.is_playing.write().await = false;
                    return Some(true); // break
                }
            }
        }
        None
    }

    // Get play speed
    async fn get_play_speed(context: &PlayContext) -> u32 {
        let speed = *context.initial_play_speed.read().await;

        if speed < 1 {
            tracing::warn!("[{}]: play speed less than 1, adjusted to 1", context.strategy_name.clone());
            1
        } else {
            speed
        }
    }

    // Handle playback finished, send playback finished event
    async fn handle_play_finished(context: &PlayContext, strategy_name: &str, play_index: PlayIndex) {
        let finish_event: BacktestStrategyEvent =
            PlayFinishedEvent::new(context.strategy_id, context.strategy_name.clone(), play_index).into();
        let _ = EventCenterSingleton::publish(finish_event.into()).await;

        tracing::info!("[{}]: kline playback finished, exiting play task normally", strategy_name);
        *context.is_playing.write().await = false;
    }

    // Play klines
    pub async fn play(&mut self) -> Result<(), BacktestStrategyError> {
        // Check if playback already finished
        let signal_generator_guard = self.signal_generator.lock().await;
        let is_finished = signal_generator_guard.is_finished();
        let current_index = signal_generator_guard.current_index();
        drop(signal_generator_guard);

        if is_finished {
            tracing::warn!("[{}]: already played finished, cannot play more kline", self.strategy_name());
            return Err(PlayFinishedSnafu {
                strategy_name: self.strategy_name().clone(),
            }
            .build());
        }

        // Check if playing state is true
        if !self.check_and_set_playing_state().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let play_context = self.create_play_context().await;
        // Indicates strategy just started, reset batch_id
        if current_index == 0 {
            self.batch_id = Uuid::new_v4();
        }

        // Update strategy status to playing
        self.store_strategy_status(BacktestStrategyRunState::Playing.to_string()).await?;

        tokio::spawn(async move {
            Self::run_play_loop(play_context).await;
        });
        Ok(())
    }

    // Pause playback
    pub async fn pause(&mut self) -> Result<(), BacktestStrategyError> {
        // Check if playing state is true
        if !*self.is_playing.read().await {
            tracing::error!("[{}]: is pausing, cannot pause again", self.strategy_name());
            return Err(AlreadyPausingSnafu {}.build());
        }
        tracing::info!("[{}]: request pause play", self.strategy_name());
        // Update strategy status to pausing
        self.store_strategy_status(BacktestStrategyRunState::Pausing.to_string()).await?;

        self.cancel_play_token.cancel();
        // Replace the cancelled token
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // Reset playback
    pub async fn reset(&mut self) -> Result<(), BacktestStrategyError> {
        // Update strategy status to ready
        self.store_strategy_status(BacktestStrategyRunState::Ready.to_string()).await?;

        // Clear logs
        self.clear_running_log().await;

        // Clear data
        self.clear_data().await;

        // Reset all custom variables
        self.reset_all_custom_variables().await;

        // Reset all system variables
        self.reset_all_sys_variables().await;

        // reset leaf node execution tracker
        self.reset_leaf_node_execution_info();

        // Reset strategy performance report
        {
            let mut benchmark_guard = self.benchmark().write().await;
            benchmark_guard.reset();
        }

        self.cancel_play_token.cancel();
        // Reset playing state
        *self.is_playing.write().await = false;

        self.cycle_watch_tx().send(Cycle::Reset).unwrap();

        let mut signal_generator_guard = self.signal_generator.lock().await;
        signal_generator_guard.reset();

        self.vts.with_ctx_write(|ctx| ctx.reset()).await;

        self.strategy_stats().with_ctx_write(|stats| stats.clear_asset_snapshots()).await;

        self.send_reset_node_event().await?;
        // Replace the cancelled token
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // Check if can play one kline
    async fn can_play_one_kline(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("[{}] is playing, cannot play one kline", self.strategy_name());
            return false;
        }
        true
    }

    // Play one kline
    pub async fn play_one(&mut self) -> Result<PlayIndex, BacktestStrategyError> {
        if !self.can_play_one_kline().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let mut signal_generator_guard = self.signal_generator.lock().await;
        let is_finished = signal_generator_guard.is_finished();

        if is_finished {
            tracing::warn!("[{}] already played finished", self.strategy_name());
            return Err(PlayFinishedSnafu {
                strategy_name: self.strategy_name().clone(),
            }
            .build());
        }

        let (signal_index, signal_time) = signal_generator_guard.next().unwrap();
        let is_finished_after_next = signal_generator_guard.is_finished();
        let mut strategy_cycle_tracker = StrategyCycleTracker::new(signal_index);
        strategy_cycle_tracker.start_phase("increment play index");
        drop(signal_generator_guard);
        tracing::debug!("signal_index: {}, signal_time: {}", signal_index, signal_time);

        let before_reset_batch_id = self.batch_id;
        // Indicates strategy just started, reset batch_id
        if signal_index == 0 {
            self.batch_id = Uuid::new_v4();
            tracing::info!(
                "[{}]: play started, reset batch_id: {}, before batch_id: {}",
                self.strategy_name(),
                self.batch_id,
                before_reset_batch_id
            );
        }
        // tracing::debug!(
        //     "playing signal_index: {}, signal_time: {}, progress: {}",
        //     signal_index,
        //     signal_time,
        //     progress
        // );

        // Start single cycle logic
        self.cycle_watch_tx().send(Cycle::Id(signal_index)).unwrap();
        self.strategy_time_watch_tx().send(signal_time).unwrap();
        if is_finished_after_next {
            let finish_event: BacktestStrategyEvent =
                PlayFinishedEvent::new(self.strategy_id(), self.strategy_name().clone(), signal_index as i32).into();
            let _ = EventCenterSingleton::publish(finish_event.into()).await;

            tracing::info!("[{}]: kline played finished, exit play task", self.strategy_name());
            self.set_is_playing(false).await;
        }

        strategy_cycle_tracker.end_phase("increment play index");
        let mut cycle_tracker_guard = self.cycle_tracker().write().await;
        *cycle_tracker_guard = Some(strategy_cycle_tracker); // Share to strategy context

        Ok(signal_index as i32)
    }

    pub async fn send_reset_node_event(&self) -> Result<(), BacktestStrategyError> {
        let nodes = self.topological_sort().unwrap();
        for node in nodes {
            let (resp_tx, resp_rx) = oneshot::channel();
            let node_id = node.node_id().await;
            let node_name = node.node_name().await;
            let payload = NodeResetCmdPayload {};
            let cmd = NodeResetCommand::new(node_id, node_name.clone(), resp_tx, payload);

            self.send_node_command(cmd.into()).await?;
            let response = resp_rx.await.context(NodeCmdRespRecvFailedSnafu {
                strategy_name: self.strategy_name().clone(),
                node_name: node_name.clone(),
            })?;
            if response.is_success() {
                tracing::info!("{}: received node reset response", response.node_id());
            } else {
                tracing::error!("{}: received node reset response failed", response.node_id());
            }
        }
        Ok(())
    }
}
