// third-party
use async_trait::async_trait;
use event_center::Event;
use key::{KeyTrait, KlineKey};
use snafu::{IntoError, ResultExt};
use star_river_core::kline::Kline;
use star_river_event::backtest_strategy::node_event::kline_node_event::{
    KlineUpdateEvent, KlineUpdatePayload, TimeUpdateEvent, TimeUpdatePayload,
};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    communication::strategy::StrategyResponse,
    error::node_error::StrategyCmdRespRecvFailedSnafu,
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use tokio::sync::oneshot;

// current crate
use super::{KlineNodeContext, utils::is_cross_interval};
// workspace crate
use crate::node::node_error::kline_node_error::{BacktestStrategySnafu, GetMinIntervalFromStrategyFailedSnafu};
use crate::{
    node::{
        node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
        node_error::kline_node_error::{KlineNodeError, KlineTimestampNotEqualSnafu},
        node_event::{BacktestNodeEvent, KlineNodeEvent, StartNodeEvent},
    },
    strategy::strategy_command::{GetMinIntervalCmdPayload, GetMinIntervalCommand, UpdateKlineDataCmdPayload, UpdateKlineDataCommand},
};

impl KlineNodeContext {
    pub(super) async fn send_kline(&mut self) -> Result<(), KlineNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.cycle_id());

        // 循环处理所有交易对
        // 上一根k线的时间戳
        let mut pre_kline_timestamp = 0;

        for (symbol_key, symbol_info) in self.selected_symbol_keys.iter() {
            // 获取k线缓存值
            // 1. 如果是最小周期的symbol，则从策略中获取k线数据
            if symbol_key.interval() == self.min_interval {
                let phase_name = format!("get min interval kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                if let Err(e) = self
                    .handle_min_interval_kline(symbol_key, symbol_info, &mut pre_kline_timestamp)
                    .await
                {
                    tracing::error!(
                        node_id = %self.node_id(),
                        node_name = %self.node_name(),
                        symbol = %symbol_key.symbol(),
                        interval = %symbol_key.interval(),
                        "Failed to handle min interval kline: {}", e
                    );
                    continue;
                }
                cycle_tracker.end_phase(&phase_name);
            } else {
                let phase_name = format!("handle interpolated kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                // 2. 如果不是最小周期的symbol，使用插值算法处理
                if let Err(e) = self.handle_interpolated_kline(symbol_key, symbol_info).await {
                    tracing::error!(
                        node_id = %self.node_id(),
                        node_name = %self.node_name(),
                        symbol = %symbol_key.symbol(),
                        interval = %symbol_key.interval(),
                        "Failed to handle interpolated kline: {}", e
                    );
                }
                cycle_tracker.end_phase(&phase_name);
            }
        }
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    // 提取发送K线事件的通用方法
    fn send_kline_events(
        &self,
        symbol_info: &(i32, String),
        kline_key: &KlineKey,
        should_calculate: bool,
        kline_data: Kline,
    ) -> Result<(), KlineNodeError> {
        let generate_event = |handle_id: String| {
            let payload = KlineUpdatePayload::new(symbol_info.0.clone(), should_calculate, kline_key.clone(), kline_data.clone());
            let kline_update_event: KlineNodeEvent = KlineUpdateEvent::new_with_time(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().clone(),
                handle_id,
                self.current_time(),
                payload,
            )
            .into();
            let backtest_node_event: BacktestNodeEvent = kline_update_event.into();
            backtest_node_event
        };

        // 发送到策略输出handle
        let strategy_output_handle = self.strategy_bound_handle();
        let event = generate_event(strategy_output_handle.output_handle_id().clone());
        self.strategy_bound_handle_send(event)?;

        let symbol_handle_id = symbol_info.1.clone();
        if self.is_leaf_node() {
            self.send_execute_over_event(Some(symbol_info.0), Some(self.current_time()))?;
        } else {
            let event = generate_event(symbol_handle_id.clone());
            self.output_handle_send(event)?;
        }

        //
        let default_output_handle = self.default_output_handle()?;
        let event = generate_event(default_output_handle.output_handle_id().clone());
        self.default_output_handle_send(event)?;
        Ok(())
    }

    // 处理插值算法的独立方法
    async fn handle_interpolated_kline(&self, symbol_key: &KlineKey, symbol_info: &(i32, String)) -> Result<(), KlineNodeError> {
        // 克隆kline_key，并设置为最小周期
        let mut kline_key_clone = symbol_key.clone();
        kline_key_clone.interval = self.min_interval.clone();

        // 从策略中获取k线数据
        let min_interval_kline_data = self
            .get_single_kline_from_strategy(&kline_key_clone, Some(self.cycle_id() as i32))
            .await?;

        // 判断当前play_index
        if self.cycle_id() == 0 {
            // 如果play_index为0，则向缓存引擎插入新的k线
            self.insert_new_kline_to_strategy(symbol_key, symbol_info, &min_interval_kline_data)
                .await
        } else {
            // 核心步骤（插值算法）
            let current_interval = symbol_key.interval();
            let is_cross_interval = is_cross_interval(&current_interval, &min_interval_kline_data.datetime());

            if is_cross_interval {
                // 如果当前是新的周期，则向缓存引擎插入新的k线
                self.insert_new_kline_to_strategy(symbol_key, symbol_info, &min_interval_kline_data)
                    .await
            } else {
                // 如果当前不是新的周期，则更新缓存引擎中的值
                self.update_existing_kline(symbol_key, symbol_info, &min_interval_kline_data).await
            }
        }
    }

    // 插入新K线到策略
    async fn insert_new_kline_to_strategy(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        min_interval_kline: &Kline,
    ) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let update_paylod = UpdateKlineDataCmdPayload::new(symbol_key.clone(), min_interval_kline.clone());
        let update_cmd = UpdateKlineDataCommand::new(self.node_id().clone(), resp_tx, update_paylod);

        self.send_strategy_command(update_cmd.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;

        match response {
            StrategyResponse::Success { .. } => {
                // 发送K线事件
                self.send_kline_events(symbol_info, symbol_key, true, min_interval_kline.clone())?;
                Ok(())
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(BacktestStrategySnafu {}.into_error(error));
            }
        }
    }

    // 更新现有K线在缓存引擎中的值
    async fn update_existing_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        min_interval_kline: &Kline,
    ) -> Result<(), KlineNodeError> {
        let last_kline = self.get_single_kline_from_strategy(symbol_key, None).await?;
        // 最小间隔k线当前的开盘价，收盘价，最高价，最低价
        let min_interval_close = min_interval_kline.close();
        let min_interval_high = min_interval_kline.high();
        let min_interval_low = min_interval_kline.low();
        let min_interval_volume = min_interval_kline.volume();

        // 计算当前k线的开盘价，收盘价，最高价，最低价
        let new_high = last_kline.high().max(min_interval_high);
        let new_low = last_kline.low().min(min_interval_low);
        let new_kline = Kline::new(
            last_kline.datetime(),                     // 时间必须和last_kline的时间一致，因为是基于last_kline的更新
            last_kline.open(),                         // 相同的时间的开盘价相同
            new_high,                                  // 最高价
            new_low,                                   // 最低价
            min_interval_close,                        // 收盘价
            last_kline.volume() + min_interval_volume, // 成交量累计
        );

        // 更新到缓存引擎
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = UpdateKlineDataCmdPayload::new(symbol_key.clone(), new_kline.clone());
        let update_cache_params = UpdateKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(update_cache_params.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;

        match response {
            StrategyResponse::Success { .. } => {
                // 使用通用方法发送K线事件
                self.send_kline_events(symbol_info, symbol_key, true, new_kline)?;
                Ok(())
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(BacktestStrategySnafu {}.into_error(error));
            }
        }
    }

    // handle min interval kline (get kline from strategy)
    async fn handle_min_interval_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        pre_kline_timestamp: &mut i64,
    ) -> Result<(), KlineNodeError> {
        let kline = self
            .get_single_kline_from_strategy(symbol_key, Some(self.cycle_id() as i32))
            .await?;
        let kline_timestamp = kline.datetime().timestamp_millis();

        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳为0， 初始值，则发送时间更新事件
        if *pre_kline_timestamp != kline_timestamp && *pre_kline_timestamp == 0 {
            *pre_kline_timestamp = kline_timestamp;

            let kline_datetime = kline.datetime();
            let payload = TimeUpdatePayload::new(kline_datetime);
            let time_update_event: KlineNodeEvent = TimeUpdateEvent::new(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().clone(),
                self.node_id().clone(),
                payload,
            )
            .into();
            self.strategy_bound_handle().send(time_update_event.into())?;
        }
        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳不为0，说明有错误，同一批k线的时间戳不一致
        else if *pre_kline_timestamp != kline_timestamp && *pre_kline_timestamp != 0 {
            return Err(KlineTimestampNotEqualSnafu {
                node_name: self.node_name().clone(),
                kline_key: symbol_key.key_str(),
                play_index: Some(self.cycle_id() as i32),
            }
            .build());
        }

        // 使用通用方法发送K线事件
        self.send_kline_events(symbol_info, symbol_key, false, kline.clone())?;

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
    type Error = KlineNodeError;

    async fn handle_command(&mut self, node_command: Self::NodeCommand) -> Result<(), KlineNodeError> {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    async fn handle_source_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), KlineNodeError> {
        match node_event {
            BacktestNodeEvent::StartNode(start_node_event) => match start_node_event {
                StartNodeEvent::KlinePlay(_) => {
                    // tracing::info!("{}: 收到KlinePlay事件: {:?}", self.node_id(), play_event);
                    self.send_kline().await
                }
            },
            _ => Ok(()),
        }
    }

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), KlineNodeError> {
        Ok(())
    }
}
