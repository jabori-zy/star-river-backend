use async_trait::async_trait;
use event_center::{CmdRespRecvFailedSnafu, Event, EventCenterSingleton};
use event_center_core::communication::Response;
use key::{IndicatorKey, KeyTrait, KlineKey};
use snafu::{IntoError, ResultExt};
use star_river_core::kline::Kline;
use star_river_event::{
    backtest_strategy::node_event::{
        IndicatorNodeEvent, KlineNodeEvent,
        indicator_node_event::{IndicatorUpdateEvent, IndicatorUpdatePayload},
        kline_node_event::KlineUpdateEvent,
    },
    communication::{CalculateIndicatorCmdPayload, CalculateIndicatorCommand, IndicatorEngineCommand},
};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategySnafu},
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use ta_lib::{Indicator, IndicatorConfig};
use tokio::sync::oneshot;

use super::IndicatorNodeContext;
use crate::{
    node::{
        node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
        node_error::{
            IndicatorNodeError,
            indicator_node_error::{CalculateIndicatorFailedSnafu, CalculateResultEmptySnafu},
        },
        node_event::BacktestNodeEvent,
    },
    strategy::strategy_command::{
        GetMinIntervalCmdPayload, GetMinIntervalCommand, UpdateIndicatorDataCmdPayload, UpdateIndicatorDataCommand,
    },
};

#[async_trait]
impl NodeEventHandlerExt for IndicatorNodeContext {
    type EngineEvent = Event;

    async fn handle_command(&mut self, node_command: Self::NodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    self.cache_kline_slice.clear();
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
            BacktestNodeEvent::KlineNode(kline_event) => {
                let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event;
                let config_kline = &self.node_config.exchange_mode()?.selected_symbol;
                if config_kline.symbol != kline_update_event.kline_key.symbol()
                    || config_kline.interval != kline_update_event.kline_key.interval()
                {
                    return Ok(());
                }
                self.handle_kline_update_event(kline_update_event).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl IndicatorNodeContext {
    /// Utility method to send indicator update event
    async fn handle_event_send(
        &self,
        handle_id: String,
        indicator_key: &IndicatorKey,
        config_id: &i32,
        indicator: Option<Indicator>,
        node_id: &String,
        node_name: &String,
        context: Option<String>,
    ) -> Result<(), IndicatorNodeError> {
        if let Some(ind) = indicator {
            // Event generation closure
            let generate_event = |target_handle_id: String| {
                let payload = IndicatorUpdatePayload::new(
                    indicator_key.exchange(),
                    indicator_key.symbol(),
                    indicator_key.interval(),
                    config_id.clone(),
                    indicator_key.get_indicator_config(),
                    indicator_key.clone(),
                    ind.clone(),
                );
                let indicator_update_event: IndicatorNodeEvent = IndicatorUpdateEvent::new_with_time(
                    self.cycle_id(),
                    node_id.clone(),
                    node_name.clone(),
                    target_handle_id,
                    self.strategy_time(),
                    payload,
                )
                .into();
                indicator_update_event.into()
            };

            let strategy_output_handle = self.strategy_bound_handle();
            let event = generate_event(strategy_output_handle.output_handle_id().clone());
            self.strategy_bound_handle_send(event)?;

            if self.is_leaf_node() {
                self.send_execute_over_event(Some(*config_id), context, Some(self.strategy_time()))?;
            } else {
                let event = generate_event(handle_id.clone());
                self.output_handle_send(event)?;

                let default_output_handle = self.default_output_handle()?;
                let event = generate_event(default_output_handle.output_handle_id().clone());
                self.default_output_handle_send(event)?;
            }
        } else {
            if self.is_leaf_node() {
                self.send_execute_over_event(Some(*config_id), context, Some(self.strategy_time()))?;
            } else {
                self.send_trigger_event(&handle_id, *config_id, context, Some(self.strategy_time()))
                    .await?;
            }
        }

        Ok(())
    }

    // Handle kline update event
    pub(super) async fn handle_kline_update_event(&mut self, kline_update_event: KlineUpdateEvent) -> Result<(), IndicatorNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.cycle_id());

        // Extract common data
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let kline_key = kline_update_event.kline_key.clone();

        let indicator_keys = self.indicator_keys.clone();

        // If current kline key is not the minimum interval trading pair, calculate and update indicator
        if self.min_interval != kline_key.interval() {
            for (indicator_key, (config_id, output_handle_id)) in indicator_keys.iter() {
                // Start tracking calculation phase for current indicator
                let phase_name = format!("calculate indicator {}", config_id);
                cycle_tracker.start_phase(&phase_name);

                self.update_kline_slice_cache(indicator_key.clone(), kline_update_event.kline.clone())
                    .await;

                let kline_series = self.cache_kline_slice.get(indicator_key);
                let lookback = self.indicator_lookback.get(indicator_key);
                if let Some(kline_series) = kline_series
                    && let Some(lookback) = lookback
                {
                    if kline_series.len() < *lookback + 1 {
                        self.handle_event_send(
                            output_handle_id.clone(),
                            &indicator_key,
                            &config_id,
                            None,
                            &node_id,
                            &node_name,
                            Some("kline series length less than lookback".to_string()),
                        )
                        .await?;
                        cycle_tracker.end_phase(&phase_name);
                        continue;
                    }
                    let calculate_result = self
                        .request_calculate_indicator(&kline_key, kline_series, &indicator_key.indicator_config)
                        .await?;
                    // Update strategy indicator data
                    self.update_strategy_indciator_data(indicator_key, calculate_result.clone()).await?;

                    // Use utility method to send indicator update event
                    self.handle_event_send(
                        output_handle_id.clone(),
                        &indicator_key,
                        &config_id,
                        Some(calculate_result),
                        &node_id,
                        &node_name,
                        Some("calculate indicator success".to_string()),
                    )
                    .await?;

                    // End tracking for current indicator
                    cycle_tracker.end_phase(&phase_name);
                }
            }
        }
        // If current kline key is the minimum interval trading pair, get and send indicator data directly
        else {
            let indicator_keys = self.indicator_keys.clone();
            // Iterate over indicator cache keys to get indicator data from strategy
            for (indicator_key, (config_id, output_handle_id)) in indicator_keys.iter() {
                let phase_name = format!("get indicator data {}", config_id);
                cycle_tracker.start_phase(&phase_name);
                // Get indicator cache data with error handling
                let indicator_data = match self.get_indicator_from_strategy(&indicator_key).await {
                    Ok(data) => data,
                    Err(_) => continue,
                };

                // Use utility method to send indicator update event
                self.handle_event_send(
                    output_handle_id.clone(),
                    &indicator_key,
                    &config_id,
                    indicator_data,
                    &node_id,
                    &node_name,
                    Some("send indicator data directly".to_string()),
                )
                .await?;
                cycle_tracker.end_phase(&phase_name);
            }
        }

        // End cycle tracking and record to benchmark
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    pub async fn init_min_interval_from_strategy(&mut self) -> Result<(), IndicatorNodeError> {
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

    // Request indicator calculation
    pub async fn request_calculate_indicator(
        &self,
        kline_key: &KlineKey,
        kline_series: &Vec<Kline>,
        indicator_config: &IndicatorConfig,
    ) -> Result<Indicator, IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = CalculateIndicatorCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            kline_key.clone(),
            kline_series.clone(),
            indicator_config.clone(),
        );
        let cmd: IndicatorEngineCommand = CalculateIndicatorCommand::new(self.node_id().clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await?;

        let response = resp_rx.await.context(CmdRespRecvFailedSnafu {})?;
        match response {
            Response::Success { payload, .. } => {
                // calculate success, but indicator result is empty
                if let Some(indicator) = payload.indicators.last() {
                    return Ok(indicator.clone());
                } else {
                    return Err(CalculateResultEmptySnafu {
                        node_name: self.node_name().clone(),
                        indicator_config: indicator_config.clone(),
                    }
                    .build());
                }
            }
            Response::Fail { error, .. } => {
                return Err(CalculateIndicatorFailedSnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error));
            }
        }
    }

    pub async fn update_strategy_indciator_data(
        &self,
        indicator_key: &IndicatorKey,
        lastest_indicator: Indicator,
    ) -> Result<(), IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = UpdateIndicatorDataCmdPayload::new(indicator_key.clone(), lastest_indicator.clone());
        let cmd = UpdateIndicatorDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.send_strategy_command(cmd.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
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
}
