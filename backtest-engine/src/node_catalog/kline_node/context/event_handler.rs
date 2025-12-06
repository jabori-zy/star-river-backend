// third-party
use async_trait::async_trait;
use event_center::Event;
use key::{KeyTrait, KlineKey};
use snafu::{IntoError, ResultExt};
use star_river_core::kline::Kline;
use star_river_event::backtest_strategy::node_event::kline_node_event::{KlineUpdateEvent, KlineUpdatePayload};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategySnafu},
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use tokio::sync::oneshot;

// current crate
use super::{KlineNodeContext, utils::is_cross_interval};
// workspace crate
use crate::node::node_error::kline_node_error::{GetMinIntervalFromStrategyFailedSnafu, PendingUpdateKlineNotExistSnafu};
use crate::{
    node::{
        node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
        node_error::kline_node_error::KlineNodeError,
        node_event::{BacktestNodeEvent, KlineNodeEvent, StartNodeEvent},
    },
    strategy::strategy_command::{GetMinIntervalCmdPayload, GetMinIntervalCommand, UpdateKlineDataCmdPayload, UpdateKlineDataCommand},
};

impl KlineNodeContext {
    pub(super) async fn send_kline(&mut self) -> Result<(), KlineNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.cycle_id());

        let selected_symbol_keys = self.selected_symbol_keys.clone();
        // Process all trading pairs in loop
        for (symbol_key, symbol_info) in selected_symbol_keys.iter() {
            // Get kline cache value
            // 1. If it's the minimum interval symbol, get kline data from strategy
            if symbol_key.interval() == self.min_interval {
                let phase_name = format!("get min interval kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                self.handle_min_interval_kline(symbol_key, symbol_info).await?;
                cycle_tracker.end_phase(&phase_name);
            } else {
                let phase_name = format!("handle interpolated kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                // 2. If not the minimum interval symbol, use interpolation algorithm
                self.handle_interpolated_kline(symbol_key, symbol_info).await?;
                cycle_tracker.end_phase(&phase_name);
            }
        }
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    // Generic method to send kline event
    async fn handle_event_send(
        &self,
        symbol_info: &(i32, String),
        kline_key: &KlineKey,
        should_calculate: bool,
        kline_data: Option<Kline>,
        is_min_interval: bool,
        context: Option<String>,
    ) -> Result<(), KlineNodeError> {
        if let Some(k) = kline_data {
            let generate_event = |handle_id: String| {
                let payload = KlineUpdatePayload::new(
                    symbol_info.0.clone(),
                    should_calculate,
                    kline_key.clone(),
                    is_min_interval,
                    k.clone(),
                );
                let kline_update_event: KlineNodeEvent = KlineUpdateEvent::new_with_time(
                    self.cycle_id(),
                    self.node_id().clone(),
                    self.node_name().clone(),
                    handle_id,
                    self.strategy_time(),
                    payload,
                )
                .into();
                kline_update_event.into()
            };
            // Send to strategy output handle
            let strategy_output_handle = self.strategy_bound_handle();
            let event = generate_event(strategy_output_handle.output_handle_id().clone());
            self.strategy_bound_handle_send(event)?;

            let symbol_handle_id = symbol_info.1.clone();
            if self.is_leaf_node() {
                self.send_execute_over_event(Some(symbol_info.0), context, Some(self.strategy_time()))?;
            } else {
                let event = generate_event(symbol_handle_id);
                self.output_handle_send(event)?;

                let default_output_handle = self.default_output_handle()?;
                let event = generate_event(default_output_handle.output_handle_id().clone());
                self.default_output_handle_send(event)?;
            }
        } else {
            if self.is_leaf_node() {
                self.send_execute_over_event(Some(symbol_info.0), context, Some(self.strategy_time()))?;
            } else {
                self.send_trigger_event(&symbol_info.1, symbol_info.0, context, Some(self.strategy_time()))
                    .await?;
            }
        }

        //

        Ok(())
    }

    // Dedicated method for handling interpolation algorithm
    async fn handle_interpolated_kline(&mut self, symbol_key: &KlineKey, symbol_info: &(i32, String)) -> Result<(), KlineNodeError> {
        // Clone kline_key and set to minimum interval
        let mut min_interval_kline_key = symbol_key.clone();
        min_interval_kline_key.interval = self.min_interval.clone();

        // Get kline data from strategy
        let min_interval_kline = self
            .get_single_kline_from_strategy(&min_interval_kline_key, Some(self.strategy_time()))
            .await?;
        if let Some(min_interval_kline) = min_interval_kline {
            // Check current cycle_id
            if self.cycle_id() == 0 {
                // If cycle_id is 0, insert new kline to cache engine
                self.insert_new_kline_to_strategy(symbol_key, &min_interval_kline).await?;
                self.handle_event_send(
                    symbol_info,
                    symbol_key,
                    true,
                    Some(min_interval_kline.clone()),
                    false,
                    Some("insert first new kline".to_string()),
                )
                .await
            } else {
                // Core step (interpolation algorithm)
                let current_interval = symbol_key.interval();
                let is_cross_interval = is_cross_interval(&current_interval, &min_interval_kline.datetime());

                if is_cross_interval {
                    // If current is a new period, insert new kline to cache engine
                    self.insert_new_kline_to_strategy(symbol_key, &min_interval_kline).await?;
                    // Send kline event
                    self.handle_event_send(
                        symbol_info,
                        symbol_key,
                        true,
                        Some(min_interval_kline.clone()),
                        false,
                        Some("insert cross interval new kline".to_string()),
                    )
                    .await
                } else {
                    // If current is not a new period, update value in cache engine
                    let last_kline = self.get_single_kline_from_strategy(symbol_key, None).await?;
                    if let Some(last_kline) = last_kline {
                        let new_kline = self.update_existing_kline(&last_kline, symbol_key, &min_interval_kline).await?;
                        // Send kline event
                        self.handle_event_send(
                            symbol_info,
                            symbol_key,
                            true,
                            Some(new_kline),
                            false,
                            Some("update existing kline".to_string()),
                        )
                        .await
                    } else {
                        return Err(PendingUpdateKlineNotExistSnafu {
                            symbol: symbol_key.symbol().to_string(),
                            interval: symbol_key.interval().to_string(),
                        }
                        .build());
                    }
                }
            }
        } else {
            self.handle_event_send(
                symbol_info,
                symbol_key,
                false,
                None,
                false,
                Some("no min interval kline".to_string()),
            )
            .await?;
            Ok(())
        }
    }

    // Insert new kline to strategy
    async fn insert_new_kline_to_strategy(&self, symbol_key: &KlineKey, interpolated_kline: &Kline) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let update_paylod = UpdateKlineDataCmdPayload::new(symbol_key.clone(), interpolated_kline.clone());
        let update_cmd = UpdateKlineDataCommand::new(self.node_id().clone(), resp_tx, update_paylod);

        self.send_strategy_command(update_cmd.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;

        match response {
            StrategyResponse::Success { .. } => {
                // Send kline event
                // self.handle_event_send(symbol_info, symbol_key, true, Some(min_interval_kline.clone())).await?;
                Ok(())
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }

    // Update existing kline value in cache engine
    async fn update_existing_kline(
        &self,
        pending_update_kline: &Kline, // The kline to be updated
        symbol_key: &KlineKey,
        min_interval_kline: &Kline,
    ) -> Result<Kline, KlineNodeError> {
        // Current open, close, high, low of minimum interval kline
        let min_interval_close = min_interval_kline.close();
        let min_interval_high = min_interval_kline.high();
        let min_interval_low = min_interval_kline.low();
        let min_interval_volume = min_interval_kline.volume();

        // Calculate new kline's open, close, high, low
        let new_high = pending_update_kline.high().max(min_interval_high);
        let new_low = pending_update_kline.low().min(min_interval_low);
        let new_kline = Kline::new(
            pending_update_kline.datetime(), // Time must match last_kline's time since it's based on last_kline update
            pending_update_kline.open(),     // Same time has same open price
            new_high,                        // High price
            new_low,                         // Low price
            min_interval_close,              // Close price
            pending_update_kline.volume() + min_interval_volume, // Volume accumulation
        );

        // Update to cache engine
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = UpdateKlineDataCmdPayload::new(symbol_key.clone(), new_kline.clone());
        let update_cache_params = UpdateKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(update_cache_params.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;

        match response {
            StrategyResponse::Success { .. } => Ok(new_kline),
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }

    // handle min interval kline (get kline from strategy)
    async fn handle_min_interval_kline(
        &mut self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String), // (config_id, handle_id)
    ) -> Result<(), KlineNodeError> {
        let kline = self.get_single_kline_from_strategy(symbol_key, Some(self.strategy_time())).await?;
        self.handle_event_send(
            symbol_info,
            symbol_key,
            false,
            kline,
            true,
            Some("handle min interval kline".to_string()),
        )
        .await?;

        Ok(())
    }

    pub async fn init_min_interval(&mut self) -> Result<(), KlineNodeError> {
        let (tx, rx) = oneshot::channel();
        let payload = GetMinIntervalCmdPayload {};
        let cmd = GetMinIntervalCommand::new(self.node_id().clone(), tx, payload);

        self.send_strategy_command(cmd.into()).await?;

        let response = rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { payload, .. } => {
                self.set_min_interval(payload.interval);
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(GetMinIntervalFromStrategyFailedSnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error));
            }
        }
    }
}

#[async_trait]
impl NodeEventHandlerExt for KlineNodeContext {
    type EngineEvent = Event;

    async fn handle_command(&mut self, node_command: Self::NodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), self.node_name().clone(), payload);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }

    async fn handle_source_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), Self::Error> {
        match node_event {
            BacktestNodeEvent::StartNode(start_node_event) => match start_node_event {
                StartNodeEvent::KlinePlay(_) => {
                    // tracing::info!("{}: Received KlinePlay event: {:?}", self.node_id(), play_event);
                    self.send_kline().await
                }
            },
            _ => Ok(()),
        }
    }

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), Self::Error> {
        Ok(())
    }
}
