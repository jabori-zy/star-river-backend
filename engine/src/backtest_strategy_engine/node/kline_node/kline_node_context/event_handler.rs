use super::{
    KlineNodeContext, CycleTracker
};
use super::utils::is_cross_interval;
use crate::backtest_strategy_engine::node::node_context::BacktestNodeContextTrait;
use crate::backtest_strategy_engine::node::node_handles::NodeOutputHandle;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::{
    GetKlineDataCmdPayload, GetKlineDataCommand, GetMinIntervalSymbolsCommand, UpdateKlineDataCmdPayload, UpdateKlineDataCommand,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::{KlineNodeEvent, TimeUpdateEvent, TimeUpdatePayload};
use event_center::event::node_event::backtest_node_event::start_node_event::KlinePlayEvent;
use snafu::Report;
use star_river_core::custom_type::PlayIndex;
use star_river_core::error::engine_error::node_error::KlineNodeError;
use star_river_core::error::engine_error::node_error::kline_node_error::{
    GetPlayKlineDataFailedSnafu, KlineTimestampNotEqualSnafu, NoMinIntervalSymbolSnafu,
};
use star_river_core::key::KeyTrait;
use star_river_core::key::key::KlineKey;
use star_river_core::market::Kline;
use star_river_core::market::QuantData;
use tokio::sync::oneshot;

impl KlineNodeContext {
    pub(super) async fn send_kline(&mut self, play_event: KlinePlayEvent) {
        let mut cycle_tracker = CycleTracker::new(self.get_play_index() as u32);
        // 提前获取配置信息，统一错误处理
        let exchange_mode_config = self.node_config.exchange_mode_config.as_ref().unwrap();

        // 获取当前play_index
        let current_play_index = play_event.play_index;

        // 循环处理所有交易对
        // 上一根k线的时间戳
        let mut pre_kline_timestamp = 0;

        for (index, (symbol_key, symbol_info)) in self.selected_symbol_keys.iter().enumerate() {
            
            // 获取k线缓存值
            // 1. 如果是在最小周期交易对列表中，则从策略中获取k线数据
            if self.min_interval_symbols.contains(symbol_key) {
                let phase_name = format!("get min interval kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                if let Err(e) = self
                    .handle_min_interval_kline(symbol_key, symbol_info, current_play_index, &mut pre_kline_timestamp)
                    .await
                {
                    tracing::error!(
                        node_id = %self.base_context.node_id,
                        node_name = %self.base_context.node_name,
                        symbol = %symbol_key.get_symbol(),
                        interval = %symbol_key.get_interval(),
                        "Failed to handle min interval kline: {}", e
                    );
                    continue;
                }
                cycle_tracker.end_phase(&phase_name);
            } else {
                let phase_name = format!("handle interpolated kline {}", symbol_info.0);
                cycle_tracker.start_phase(&phase_name);
                // 2. 如果不在最小周期交易对列表中，使用插值算法处理
                if let Err(e) = self.handle_interpolated_kline(symbol_key, symbol_info, current_play_index).await {
                    tracing::error!(
                        node_id = %self.base_context.node_id,
                        node_name = %self.base_context.node_name,
                        symbol = %symbol_key.get_symbol(),
                        interval = %symbol_key.get_interval(),
                        "Failed to handle interpolated kline: {}", e
                    );
                }
                cycle_tracker.end_phase(&phase_name);
            }

            
            if index == exchange_mode_config.selected_symbols.len() - 1 {
                if self.is_leaf_node() {
                    self.send_execute_over_event().await;
                }
            }
            
        }
        let completed_tracker = cycle_tracker.end();
        self.add_node_cycle_tracker(self.get_node_id().clone(), completed_tracker).await;
    }

    // 提取发送K线事件的通用方法
    fn send_kline_events(
        &self,
        symbol_info: &(i32, String),
        kline_key: &KlineKey,
        should_calculate: bool,
        play_index: PlayIndex,
        kline_data: Kline,
    ) {
        let send_kline_event = |handle_id: String, output_handle: &NodeOutputHandle| {
            let kline_update_event = self.get_kline_update_event(
                handle_id,
                symbol_info.0.clone(),
                should_calculate,
                kline_key,
                play_index,
                kline_data.clone(),
            );
            let kline_node_event = BacktestNodeEvent::KlineNode(kline_update_event);
            let _ = output_handle.send(kline_node_event);
        };

        // 发送到交易对特定的输出handle
        let symbol_handle_id = symbol_info.1.clone();
        let symbol_output_handle = self.get_output_handle(&symbol_handle_id);
        tracing::debug!("symbol_output_handle: {}", symbol_output_handle);
        tracing::debug!("strategy_output_handle: {}", self.get_strategy_output_handle());
        if symbol_output_handle.is_connected() {
            send_kline_event(symbol_handle_id, symbol_output_handle);
        }

        // 发送到默认输出handle
        let default_output_handle = self.get_default_output_handle();
        if default_output_handle.is_connected() {
            send_kline_event(default_output_handle.output_handle_id(), default_output_handle);
        }

        // 发送到策略输出handle
        let strategy_output_handle = self.get_strategy_output_handle();
        send_kline_event(strategy_output_handle.output_handle_id(), strategy_output_handle);
    }

    // 处理插值算法的独立方法
    async fn handle_interpolated_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
    ) -> Result<(), KlineNodeError> {
        // 先找到相同symbol的min_interval_symbol
        let min_interval_symbol = self
            .min_interval_symbols
            .iter()
            .find(|k| k.get_symbol() == symbol_key.get_symbol())
            .ok_or_else(|| {
                NoMinIntervalSymbolSnafu {
                    symbol: symbol_key.get_symbol().clone(),
                }
                .build()
            })?;

        // 从策略中获取k线数据
        let min_interval_kline_data = self.get_kline_from_strategy(min_interval_symbol, current_play_index).await.map_err(|e| {
            tracing::error!(
                node_id = %self.base_context.node_id,
                node_name = %self.base_context.node_name,
                "Failed to get history kline cache: {}", e
            );
            e
        })?;

        // 判断当前play_index
        if current_play_index == 0 {
            // 如果play_index为0，则向缓存引擎插入新的k线
            self.insert_new_kline_to_strategy(
                symbol_key,
                symbol_info,
                current_play_index,
                &min_interval_kline_data.last().unwrap(),
            )
            .await
        } else {
            // 核心步骤（插值算法）
            let current_interval = symbol_key.get_interval();
            let is_cross_interval = is_cross_interval(&current_interval, &min_interval_kline_data.last().unwrap().get_datetime());

            if is_cross_interval {
                // 如果当前是新的周期，则向缓存引擎插入新的k线
                self.insert_new_kline_to_strategy(
                    symbol_key,
                    symbol_info,
                    current_play_index,
                    &min_interval_kline_data.last().unwrap(),
                )
                .await
            } else {
                // 如果当前不是新的周期，则更新缓存引擎中的值
                self.update_existing_kline(
                    symbol_key,
                    symbol_info,
                    current_play_index,
                    &min_interval_kline_data.last().unwrap(),
                )
                .await
            }
        }
    }

    // 插入新K线到策略
    async fn insert_new_kline_to_strategy(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
        min_interval_kline: &Kline,
    ) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let update_paylod = UpdateKlineDataCmdPayload::new(symbol_key.clone(), min_interval_kline.clone());
        let update_cmd = UpdateKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(update_paylod));

        let _ = self.get_strategy_command_sender().send(update_cmd.into()).await;
        let response = resp_rx.await.unwrap();

        if response.is_success() {
            // 发送K线事件
            self.send_kline_events(symbol_info, symbol_key, true, current_play_index, min_interval_kline.clone());
            Ok(())
        } else {
            let error = response.get_error();
            tracing::error!("{}", Report::from_error(error));
            return Ok(());
        }
    }

    // 更新现有K线在缓存引擎中的值
    async fn update_existing_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
        min_interval_kline: &Kline,
    ) -> Result<(), KlineNodeError> {
        // 先获取缓存引擎中的最后一个值
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(symbol_key.clone(), None, Some(1));
        let get_last_kline_cmd = GetKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));
        let _ = self.get_strategy_command_sender().send(get_last_kline_cmd.into()).await;
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            let last_kline = response.kline_series.last().unwrap();

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
            let update_cache_params = UpdateKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));
            let _ = self.get_strategy_command_sender().send(update_cache_params.into()).await;
            let response = resp_rx.await.unwrap();

            if response.is_success() {
                // 使用通用方法发送K线事件
                self.send_kline_events(symbol_info, symbol_key, true, current_play_index, new_kline);
                Ok(())
            } else {
                let error = response.get_error();
                tracing::error!("{}", Report::from_error(error));
                return Ok(());
            }
        } else {
            return Err(GetPlayKlineDataFailedSnafu {
                node_name: self.get_node_name().clone(),
                kline_key: symbol_key.get_key_str(),
                play_index: current_play_index as u32,
            }
            .fail()?);
        }
    }

    // 处理最小周期K线
    async fn handle_min_interval_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
        pre_kline_timestamp: &mut i64,
    ) -> Result<(), KlineNodeError> {
        let kline = self.get_kline_from_strategy(symbol_key, current_play_index).await?;
        let kline_timestamp = kline.last().unwrap().get_datetime().timestamp_millis();

        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳为0， 初始值，则发送时间更新事件
        if *pre_kline_timestamp != kline_timestamp && *pre_kline_timestamp == 0 {
            *pre_kline_timestamp = kline_timestamp;

            let kline_datetime = kline.last().unwrap().get_datetime();
            let payload = TimeUpdatePayload::new(kline_datetime);
            let time_update_event: KlineNodeEvent = TimeUpdateEvent::new(
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                self.get_node_id().clone(),
                payload,
            )
            .into();
            self.get_strategy_output_handle().send(time_update_event.into()).unwrap();
        }
        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳不为0，说明有错误，同一批k线的时间戳不一致
        else if *pre_kline_timestamp != kline_timestamp && *pre_kline_timestamp != 0 {
            tracing::error!(
                node_id = %self.base_context.node_id,
                node_name = %self.base_context.node_name,
                "kline timestamp is not equal to previous kline timestamp"
            );
            return Err(KlineTimestampNotEqualSnafu {
                node_name: self.base_context.node_name.clone(),
                kline_key: symbol_key.get_key_str(),
                play_index: current_play_index as u32,
            }
            .fail()?);
        }

        // 使用通用方法发送K线事件
        self.send_kline_events(symbol_info, symbol_key, false, current_play_index, kline.last().unwrap().clone());

        Ok(())
    }

    pub async fn get_min_interval_symbols(&mut self) -> Result<Vec<KlineKey>, String> {
        let (tx, rx) = oneshot::channel();
        let cmd = GetMinIntervalSymbolsCommand::new(self.get_node_id().clone(), tx, None);

        self.get_strategy_command_sender().send(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.keys.clone());
        } else {
            return Err("获取最小周期交易对失败".to_string());
        }
    }
}
