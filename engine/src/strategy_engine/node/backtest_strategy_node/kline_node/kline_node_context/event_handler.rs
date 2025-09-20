use super::KlineNodeContext;
use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, TimeUpdateEvent, TimeUpdatePayload,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::start_node_event::KlinePlayEvent;
use star_river_core::cache::Key;
use event_center::communication::strategy::NodeResponse;
use event_center::communication::strategy::backtest_strategy::command::GetMinIntervalSymbolsParams;
use event_center::communication::strategy::backtest_strategy::response::BacktestNodeResponse;
use star_river_core::market::Kline;
use tokio::sync::oneshot;
use super::utils::is_cross_interval;
use event_center::communication::engine::cache_engine::{GetCacheParams, UpdateCacheParams};
use event_center::EventCenterSingleton;
use event_center::communication::engine::cache_engine::CacheEngineResponse;
use event_center::communication::engine::EngineResponse;
use std::sync::Arc;
use star_river_core::cache::CacheValue;
use star_river_core::custom_type::PlayIndex;


impl KlineNodeContext {
    // 提取发送K线事件的通用方法
    fn send_kline_events(
        &self,
        symbol_info: &(i32, String),
        symbol_key: &Key,
        play_index: PlayIndex,
        kline_data: Vec<Arc<CacheValue>>,
    ) {
        let send_kline_event = |handle_id: String, output_handle: &NodeOutputHandle| {
            let kline_update_event = self.get_kline_update_event(
                handle_id,
                symbol_info.0.clone(),
                symbol_key,
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
            send_kline_event(
                default_output_handle.output_handle_id.clone(),
                default_output_handle,
            );
        }

        // 发送到策略输出handle
        let strategy_output_handle = self.get_strategy_output_handle();
        send_kline_event(
            strategy_output_handle.output_handle_id.clone(),
            strategy_output_handle,
        );
    }

    pub(super) async fn send_kline(&self, play_event: KlinePlayEvent) {
        // 提前获取配置信息，统一错误处理
        let exchange_config = self.backtest_config.exchange_mode_config.as_ref().unwrap();

        let current_play_index = self.get_play_index();

        // 如果索引不匹配，提前返回错误日志
        if current_play_index != play_event.play_index {
            tracing::error!(
                node_id = %self.base_context.node_id,
                node_name = %self.base_context.node_name,
                kline_cache_index = %play_event.play_index,
                signal_index = %current_play_index,
                "kline cache index is not equal to signal index"
            );
            return;
        }



        // 循环处理所有交易对
        // 上一根k线的时间戳
        let mut pre_kline_timestamp = 0;
        for (index, (symbol_key, symbol_info)) in self.selected_symbol_keys.iter().enumerate() {

            // 获取k线缓存值
            // 1. 如果是在最小周期交易对列表中，则从缓存引擎获取k线数据
            if self.min_interval_symbols.contains(symbol_key) {
                let kline_cache_value = self.get_history_kline_cache(symbol_key, current_play_index).await;
                let kline_cache_value = match kline_cache_value {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::error!(
                            node_id = %self.base_context.node_id,
                            node_name = %self.base_context.node_name,
                            "Failed to get history kline cache: {}", e
                        );
                        continue;
                    }
                };
                let kline_timestamp = kline_cache_value.last().unwrap().get_timestamp();
                // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳为0， 初始值，则发送时间更新事件
                if pre_kline_timestamp != kline_timestamp && pre_kline_timestamp == 0 {
                    pre_kline_timestamp = kline_timestamp;
                    let kline_datetime = kline_cache_value.last().unwrap().get_datetime();
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
                else if pre_kline_timestamp != kline_timestamp
                    && pre_kline_timestamp != 0
                {
                    tracing::error!(
                        node_id = %self.base_context.node_id,
                        node_name = %self.base_context.node_name,
                        "kline timestamp is not equal to previous kline timestamp"
                    );
                    continue;
                }

                // 使用通用方法发送K线事件
                self.send_kline_events(symbol_info, &symbol_key, current_play_index, kline_cache_value.clone());

                if index == exchange_config.selected_symbols.len() - 1 {
                    if self.is_leaf_node() {
                        self.send_execute_over_event().await;
                    }
                }

            } else {
                // 2. 如果不在最小周期交易对列表中，则先打印警告日志
                // tracing::warn!("[{}] symbol: {}-{}, is not min interval, skip", self.get_node_name(), symbol_key.get_symbol(), symbol_key.get_interval());
                // 1. 先找到相同symbol的min_interval_symbol
                let min_interval_symbol = self.min_interval_symbols.iter().find(|k| k.get_symbol() == symbol_key.get_symbol()).unwrap();
                // 2. 从缓存引擎获取k线数据
                let kline_cache_value = self.get_history_kline_cache(min_interval_symbol, current_play_index).await;
                let kline_cache_value = match kline_cache_value {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::error!(
                            node_id = %self.base_context.node_id,
                            node_name = %self.base_context.node_name,
                            "Failed to get history kline cache: {}", e
                        );
                        continue;
                    }
                };

                // 3. 判断当前play_index
                if current_play_index == 0 {
                    // 3.1 如果play_index为0，则向缓存引擎插入新的k线
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let update_cache_params = UpdateCacheParams::new(
                        self.get_strategy_id().clone(),
                        symbol_key.clone(),
                        kline_cache_value[0].as_ref().clone(),
                        self.get_node_id().clone(),
                        resp_tx,
                    );
                    let _ = EventCenterSingleton::send_command(update_cache_params.into())
                        .await;
                    let response = resp_rx.await.unwrap();
                    if response.success() {
                        continue;
                    }
                } else {
                    // 4. 核心步骤（插值算法）
                    // 以1小时和1m的k线为例
                    // 如果当前需要1小时k线，但是此时只有1mk线
                    // 判断当前是否是新的小时，如果是，则向缓存引擎插入新的1小时k线，如果不是，则更新缓存引擎中的值
                    let current_interval = symbol_key.get_interval();
                    let is_cross_interval = is_cross_interval(&current_interval, &kline_cache_value.last().unwrap().get_datetime());
                    // 如果当前是新的小时，则向缓存引擎插入新的k线
                    if is_cross_interval {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let update_cache_params = UpdateCacheParams::new(
                            self.get_strategy_id().clone(),
                            symbol_key.clone(),
                            kline_cache_value[0].as_ref().clone(),
                            self.get_node_id().clone(),
                            resp_tx,
                        );
                        let _ = EventCenterSingleton::send_command(update_cache_params.into())
                            .await;
                        let response = resp_rx.await.unwrap();
                        if response.success() {
                            continue;
                        }
                    } 
                    // 如果当前不是新的小时，则更新缓存引擎中的值
                    else {
                        // 先获取缓存引擎中的最后一个值
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let get_cache_value_params = GetCacheParams::new(
                            self.get_strategy_id().clone(),
                            self.get_node_id().clone(),
                            symbol_key.clone(),
                            None,
                            Some(1),
                            self.get_node_id().clone(),
                            resp_tx,
                        );
                        let _ = EventCenterSingleton::send_command(get_cache_value_params.into())
                            .await;
                        let response = resp_rx.await.unwrap();
                        if response.success() {
                            match response {
                                EngineResponse::CacheEngine(CacheEngineResponse::GetCacheData(get_cache_data_response)) => {
                                    tracing::info!("get_cache_data_response: {:#?}", get_cache_data_response.cache_data);
                                    let last_kline = get_cache_data_response.cache_data.last().unwrap().as_kline().unwrap();
                                    // 最小间隔k线当前的开盘价，收盘价，最高价，最低价
                                    let min_interval_kline = kline_cache_value.last().unwrap().as_kline().unwrap();
                                    let min_interval_close = min_interval_kline.close();
                                    let min_interval_high = min_interval_kline.high();
                                    let min_interval_low = min_interval_kline.low();
                                    let min_interval_volume = min_interval_kline.volume();
                                    // 计算当前k线的开盘价，收盘价，最高价，最低价
                                    let new_high = last_kline.high().max(min_interval_high);
                                    let new_low = last_kline.low().min(min_interval_low);
                                    let new_kline = Kline::new(
                                        last_kline.datetime(), // 时间必须和last_kline的时间一致，因为是基于last_kline的更新
                                        last_kline.open(), // 相同的时间的开盘价相同
                                        new_high, // 最高价
                                        new_low, // 最低价
                                        min_interval_close, // 收盘价
                                        last_kline.volume() + min_interval_volume, // 成交量累计
                                    );
                                    // 更新到缓存引擎
                                    let (resp_tx, resp_rx) = oneshot::channel();
                                    let update_cache_params = UpdateCacheParams::new(
                                        self.get_strategy_id().clone(),
                                        symbol_key.clone(),
                                        new_kline.clone().into(),
                                        self.get_node_id().clone(),
                                        resp_tx,
                                    );
                                    let _ = EventCenterSingleton::send_command(update_cache_params.into())
                                        .await;
                                    let response = resp_rx.await.unwrap();
                                    if response.success() {
                                        // 使用通用方法发送K线事件
                                        self.send_kline_events(symbol_info, &symbol_key, current_play_index, vec![Arc::new(new_kline.into())]);
                                        continue;
                                    }

                                    
                                }
                                _ => {}
                            }
                        }

                    }

                    

                }
                continue;
            }
            
            

            



            
        }
    }



    pub async fn get_min_interval_symbols(&mut self) -> Result<Vec<Key>, String> {
        let (tx, rx) = oneshot::channel();
        let get_min_interval_symbols_params = GetMinIntervalSymbolsParams::new(self.get_node_id().clone(), tx);

        self.get_node_command_sender()
            .send(get_min_interval_symbols_params.into())
            .await
            .unwrap();

        let response = rx.await.unwrap();
        match response {
            NodeResponse::BacktestNode(BacktestNodeResponse::GetMinIntervalSymbols(
                get_min_interval_symbols_response,
            )) => return Ok(get_min_interval_symbols_response.keys),
            _ => return Err("获取最小周期交易对失败".to_string()),
        }
    }

}