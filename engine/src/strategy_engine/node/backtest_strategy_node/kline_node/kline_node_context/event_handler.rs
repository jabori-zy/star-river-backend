use super::KlineNodeContext;
use super::utils::is_cross_interval;
use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use event_center::EventCenterSingleton;
use event_center::communication::engine::EngineResponse;
use event_center::communication::engine::cache_engine::CacheEngineResponse;
use event_center::communication::engine::cache_engine::{GetCacheParams, UpdateCacheParams};
use event_center::communication::strategy::{GetKlineDataParams, NodeResponse, UpdateKlineDataParams};
use event_center::communication::strategy::backtest_strategy::command::GetMinIntervalSymbolsParams;
use event_center::communication::strategy::backtest_strategy::response::BacktestStrategyResponse;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, TimeUpdateEvent, TimeUpdatePayload,
};
use event_center::event::node_event::backtest_node_event::start_node_event::KlinePlayEvent;
use snafu::Report;
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::{CacheItem, CacheValue};
use star_river_core::cache::{Key, KeyTrait};
use star_river_core::custom_type::PlayIndex;
use star_river_core::error::engine_error::node_error::kline_node_error::{GetKlineDataSnafu, KlineTimestampNotEqualSnafu, NoMinIntervalSymbolSnafu};
use star_river_core::error::engine_error::node_error::KlineNodeError;
use star_river_core::market::Kline;
use std::sync::Arc;
use tokio::sync::oneshot;

impl KlineNodeContext {
    pub(super) async fn send_kline(&self, play_event: KlinePlayEvent) {
        // 提前获取配置信息，统一错误处理
        let exchange_mode_config = self.backtest_config.exchange_mode_config.as_ref().unwrap();

        // 获取当前play_index
        let current_play_index = play_event.play_index;

        // 循环处理所有交易对
        // 上一根k线的时间戳
        let mut pre_kline_timestamp = 0;

        for (index, (symbol_key, symbol_info)) in self.selected_symbol_keys.iter().enumerate() {
            // 获取k线缓存值
            // 1. 如果是在最小周期交易对列表中，则从策略中获取k线数据
            if self.min_interval_symbols.contains(symbol_key) {
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
            } else {
                // 2. 如果不在最小周期交易对列表中，使用插值算法处理
                if let Err(e) = self
                    .handle_interpolated_kline(symbol_key, symbol_info, current_play_index)
                    .await
                {
                    tracing::error!(
                        node_id = %self.base_context.node_id,
                        node_name = %self.base_context.node_name,
                        symbol = %symbol_key.get_symbol(),
                        interval = %symbol_key.get_interval(),
                        "Failed to handle interpolated kline: {}", e
                    );
                }
            }

            if index == exchange_mode_config.selected_symbols.len() - 1 {
                if self.is_leaf_node() {
                    self.send_execute_over_event().await;
                }
            }
        }
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
        if symbol_output_handle.connect_count > 0 {
            send_kline_event(symbol_handle_id, symbol_output_handle);
        }

        // 发送到默认输出handle
        let default_output_handle = self.get_default_output_handle();
        if default_output_handle.connect_count > 0 {
            send_kline_event(default_output_handle.output_handle_id.clone(), default_output_handle);
        }

        // 发送到策略输出handle
        let strategy_output_handle = self.get_strategy_output_handle();
        send_kline_event(strategy_output_handle.output_handle_id.clone(), strategy_output_handle);
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
            .ok_or_else(|| NoMinIntervalSymbolSnafu {
                symbol: symbol_key.get_symbol().clone(),
            }
            .build())?;

        // 从缓存引擎获取k线数据
        let min_interval_kline_data = self
            .get_kline(min_interval_symbol, current_play_index)
            .await
            .map_err(|e| {
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
            self.insert_new_kline(
                symbol_key,
                symbol_info,
                current_play_index,
                &min_interval_kline_data.last().unwrap(),
            )
            .await
        } else {
            // 核心步骤（插值算法）
            let current_interval = symbol_key.get_interval();
            let is_cross_interval = is_cross_interval(
                &current_interval,
                &min_interval_kline_data.last().unwrap().get_datetime(),
            );

            if is_cross_interval {
                // 如果当前是新的周期，则向缓存引擎插入新的k线
                self.insert_new_kline(
                    symbol_key,
                    symbol_info,
                    current_play_index,
                    &min_interval_kline_data.last().unwrap(),
                )
                .await
            } else {
                // 如果当前不是新的周期，则更新缓存引擎中的值
                self.update_existing_kline(symbol_key, symbol_info, current_play_index, &min_interval_kline_data.last().unwrap()).await
            }
        }
    }

    // 插入新K线到缓存引擎
    async fn insert_new_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
        min_interval_kline: &Kline,
    ) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let update_cache_params = UpdateKlineDataParams::new(
            self.get_node_id().clone(),
            symbol_key.clone(),
            min_interval_kline.clone(),
            resp_tx,
        );
        let _ = self.get_node_command_sender().send(update_cache_params.into()).await;
        let response = resp_rx.await.unwrap();

        if response.success() {
            // 发送K线事件
            self.send_kline_events(
                symbol_info,
                symbol_key,
                true,
                current_play_index,
                min_interval_kline.clone(),
            );
            Ok(())
        } else {
            let error = response.error();
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
        let get_last_kline_params = GetKlineDataParams::new(
            self.get_node_id().clone(),
            symbol_key.clone().into(),
            None,
            Some(1),
            resp_tx,
        );
        let _ = self.get_node_command_sender().send(get_last_kline_params.into()).await;
        let response = resp_rx.await.unwrap();

        if !response.success() {
            return Err(GetKlineDataSnafu {
                node_name: self.get_node_name().clone(),
                kline_key: symbol_key.get_key_str(),
                play_index: current_play_index as u32,
            }
            .fail()?);
        }

        match response {
            NodeResponse::BacktestNode(BacktestStrategyResponse::GetKlineData(resp)) => {
                let last_kline = resp.data.last().unwrap();

                // 最小间隔k线当前的开盘价，收盘价，最高价，最低价
                let min_interval_close = min_interval_kline.close();
                let min_interval_high = min_interval_kline.high();
                let min_interval_low = min_interval_kline.low();
                let min_interval_volume = min_interval_kline.volume();

                // 计算当前k线的开盘价，收盘价，最高价，最低价
                let new_high = last_kline.high().max(min_interval_high);
                let new_low = last_kline.low().min(min_interval_low);
                let new_kline = Kline::new(
                    last_kline.datetime(), // 时间必须和last_kline的时间一致，因为是基于last_kline的更新
                    last_kline.open(),     // 相同的时间的开盘价相同
                    new_high,              // 最高价
                    new_low,               // 最低价
                    min_interval_close,    // 收盘价
                    last_kline.volume() + min_interval_volume, // 成交量累计
                );

                // 更新到缓存引擎
                let (resp_tx, resp_rx) = oneshot::channel();
                let update_cache_params = UpdateKlineDataParams::new(
                    self.get_node_id().clone(),
                    symbol_key.clone(),
                    new_kline.clone().into(),
                    resp_tx,
                );
                let _ = self.get_node_command_sender().send(update_cache_params.into()).await;
                let response = resp_rx.await.unwrap();

                if response.success() {
                    // 使用通用方法发送K线事件
                    self.send_kline_events(symbol_info, symbol_key, true, current_play_index, new_kline);
                    Ok(())
                } else {
                    let error = response.error();
                    tracing::error!("{}", Report::from_error(error));
                    return Ok(());
                }
            }
            _ => {
                let error = response.error();
                tracing::error!("{}", Report::from_error(error));
                return Ok(());
            }
        }
    }

    // 处理最小周期K线的独立方法
    async fn handle_min_interval_kline(
        &self,
        symbol_key: &KlineKey,
        symbol_info: &(i32, String),
        current_play_index: PlayIndex,
        pre_kline_timestamp: &mut i64,
    ) -> Result<(), KlineNodeError> {
        let kline = self.get_kline(symbol_key, current_play_index).await?;
        let kline_timestamp = kline.last().unwrap().get_timestamp();

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
            self.get_strategy_output_handle()
                .send(time_update_event.into())
                .unwrap();
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
        let get_min_interval_symbols_params = GetMinIntervalSymbolsParams::new(self.get_node_id().clone(), tx);

        self.get_node_command_sender()
            .send(get_min_interval_symbols_params.into())
            .await
            .unwrap();

        let response = rx.await.unwrap();
        match response {
            NodeResponse::BacktestNode(BacktestStrategyResponse::GetMinIntervalSymbols(
                get_min_interval_symbols_response,
            )) => return Ok(get_min_interval_symbols_response.keys),
            _ => return Err("获取最小周期交易对失败".to_string()),
        }
    }
}
