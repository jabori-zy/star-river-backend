use super::IndicatorNodeContext;
use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use event_center::communication::backtest_strategy::GetMinIntervalSymbolsCommand;
use event_center::communication::backtest_strategy::UpdateIndicatorDataCmdPayload;
use event_center::communication::backtest_strategy::UpdateIndicatorDataCommand;
use event_center::communication::engine::indicator_engine::CalculateHistoryIndicatorCmdPayload;
use event_center::communication::engine::indicator_engine::CalculateHistoryIndicatorCommand;
use event_center::communication::engine::indicator_engine::IndicatorEngineCommand;
use event_center::EventCenterSingleton;
use event_center::communication::engine::EngineResponse;
use event_center::event::node_event::backtest_node_event::common_event::TriggerPayload;
use event_center::event::node_event::backtest_node_event::common_event::{
    CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent,
};
use event_center::event::node_event::backtest_node_event::indicator_node_event::{
    IndicatorNodeEvent, IndicatorUpdateEvent, IndicatorUpdatePayload,
};
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use star_river_core::key::key::KlineKey;
use star_river_core::key::KeyTrait;
use star_river_core::indicator::Indicator;
use tokio::sync::oneshot;
use event_center::communication::Response;

impl IndicatorNodeContext {
    /// 发送指标更新事件的工具方法
    fn send_indicator_update_event(
        &self,
        handle_id: String,
        indicator_key: &star_river_core::key::key::IndicatorKey,
        config_id: &i32,
        indicator_value: Indicator,
        play_index: i32,
        node_id: &String,
        node_name: &String,
        to_strategy: bool,
    ) {
        let payload = IndicatorUpdatePayload::new(
            indicator_key.get_exchange(),
            indicator_key.get_symbol(),
            indicator_key.get_interval(),
            config_id.clone(),
            indicator_key.get_indicator_config(),
            indicator_key.clone(),
            indicator_value,
            play_index,
        );
        let indicator_update_event: IndicatorNodeEvent =
            IndicatorUpdateEvent::new(node_id.clone(), node_name.clone(), handle_id.clone(), payload).into();

        // 发送到指标特定的输出handle（如果存在）
        if let Some(output_handle) = self.base_context.output_handles.get(&handle_id) {
            let _ = output_handle.send(indicator_update_event.clone().into());
        }

        // 发送到默认输出handle
        let default_output_handle = self.get_default_output_handle();
        let _ = default_output_handle.send(indicator_update_event.clone().into());

        // 发送到strategy
        if to_strategy {
            let strategy_output_handle = self.get_strategy_output_handle();
            let _ = strategy_output_handle.send(indicator_update_event.into());
        }
    }

    // 处理k线更新事件
    pub(super) async fn handle_kline_update(&mut self, kline_update_event: KlineNodeEvent) {
        if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_update_event {
            // 提取公共数据
            let strategy_id = self.get_strategy_id().clone();
            let node_id = self.get_node_id().clone();
            let node_name = self.get_node_name().clone();
            let kline_key = kline_update_event.kline_key.clone();
            
            let indicator_keys = self.indicator_keys.clone();

            // 如果当前k线key不是最小周期交易对，则更新指标缓存数据
            if !self.min_interval_symbols.contains(&self.selected_kline_key) {
                for (indicator_key, (config_id, output_handle_id)) in indicator_keys.iter() {
                    
                    self.update_kline_data(indicator_key.clone(), kline_update_event.kline.clone()).await;
                    
                    let kline_series = self.kline_value.get(indicator_key).unwrap();
                    let lookback = self.indicator_lookback.get(indicator_key).unwrap();
                    if kline_series.len() < *lookback + 1 {
                        tracing::warn!("指标缓存数据长度小于lookback, skip. lookback: {}, kline_series_len: {}", lookback, kline_series.len());
                        continue;
                    }
                    tracing::debug!("计算指标: {:#?}", indicator_key.indicator_config);
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let payload = CalculateHistoryIndicatorCmdPayload::new(
                        strategy_id,
                        node_id.clone(),
                        kline_key.clone(),
                        kline_series.clone(),
                        indicator_key.indicator_config.clone(),
                    );
                    let cmd: IndicatorEngineCommand = CalculateHistoryIndicatorCommand::new(
                        node_id.clone(),
                        resp_tx,
                        Some(payload),
                    ).into();

                    let _ = EventCenterSingleton::send_command(cmd.into()).await;
                    let response = resp_rx.await.unwrap();
                    if response.is_success() {
                        let indicator_data = response.indicators.clone();
                        // 更新指标
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let last_indicator = indicator_data.last().unwrap();
                        let payload = UpdateIndicatorDataCmdPayload::new(
                            indicator_key.clone(), 
                            last_indicator.clone(),
                        );
                        let cmd = UpdateIndicatorDataCommand::new(
                            node_id.clone(), 
                            resp_tx,
                            Some(payload),
                        );
                        self.get_strategy_command_sender().send(cmd.into()).await.unwrap();
                        let response = resp_rx.await.unwrap();
                        if response.is_success() {
                            // 使用工具方法发送指标更新事件
                            self.send_indicator_update_event(
                                output_handle_id.clone(),
                                &indicator_key,
                                &config_id,
                                last_indicator.clone(),
                                kline_update_event.play_index,
                                &node_id,
                                &node_name,
                                true,
                            );
                        }
                    } else {
                        // 发送触发事件
                        let payload = TriggerPayload::new(self.get_play_index());
                        let trigger_event: CommonEvent =
                            TriggerEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload)
                                .into();
                        let _ = self.get_output_handle(output_handle_id).send(trigger_event.into());
                    }
                }
            } 
            // 如果当前k线key是最小周期交易对，则直接发送指标更新事件
            else {
                // 遍历指标缓存键，从策略中获取指标数据
                for (indicator_key, (config_id, output_handle_id)) in self.indicator_keys.iter() {
                    // 获取指标缓存数据，增加错误处理
                    let indicator_cache_data = match self
                        .get_indicator_data(&indicator_key, kline_update_event.play_index)
                        .await
                    {
                        Ok(data) => data,

                        Err(e) => {
                            tracing::error!(
                                node_id = %self.base_context.node_id,
                                node_name = %self.base_context.node_name,
                                indicator = ?indicator_key.indicator_config,
                                "Failed to get backtest indicator cache: {}", e
                            );
                            continue;
                        }
                    };

                    // 使用工具方法发送指标更新事件
                    self.send_indicator_update_event(
                        output_handle_id.clone(),
                        &indicator_key,
                        &config_id,
                        indicator_cache_data,
                        kline_update_event.play_index,
                        &node_id,
                        &node_name,
                        true,
                    );
                }
            }

            // 如果节点是叶子节点，则发送执行完毕事件
            if self.is_leaf_node() {
                let payload = ExecuteOverPayload::new(self.get_play_index());
                let execute_over_event: CommonEvent = ExecuteOverEvent::new(
                    self.get_node_id().clone(),
                    self.get_node_name().clone(),
                    self.get_node_id().clone(),
                    payload,
                )
                .into();
                let strategy_output_handle = self.get_strategy_output_handle();
                strategy_output_handle.send(execute_over_event.into()).unwrap();
            }
        }
    }

    pub async fn get_min_interval_symbols(&mut self) -> Result<Vec<KlineKey>, String> {
        let (tx, rx) = oneshot::channel();
        let cmd = GetMinIntervalSymbolsCommand::new(self.get_node_id().clone(), tx, None);

        self.get_strategy_command_sender()
            .send(cmd.into())
            .await
            .unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.keys.clone());
        }
        else {
            return Err("获取最小周期交易对失败".to_string());
        }
    }

    // 节点重置
    pub(super) async fn handle_node_reset(&self) {
        // 将缓存引擎中的，不在min_interval_symbols中的指标缓存键删除
        // if !self.min_interval_symbols.contains(&self.selected_kline_key) {
        //     for (indicator_key, _) in self.indicator_keys.iter() {
        //         let (resp_tx, resp_rx) = oneshot::channel();
        //         let 
        //         let clear_cache_params = ClearCacheParams::new(
        //             self.get_strategy_id().clone(),
        //             indicator_key.clone().into(),
        //             self.get_node_id().clone(),
        //             resp_tx,
        //         );
        //         let _ = EventCenterSingleton::send_command(clear_cache_params.into()).await;
        //         let response = resp_rx.await.unwrap();
        //         if response.success() {
        //             tracing::debug!("删除指标缓存成功");
        //         } else {
        //             tracing::error!("删除指标缓存失败: {:#?}", response);
        //         }
        //     }
        // } else {
        //     tracing::debug!("节点重置，无需删除指标缓存");
        // }
    }
}
