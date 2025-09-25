mod context_impl;
mod event_handler;
mod status_handler;
mod data_handler;

use super::indicator_node_type::IndicatorNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use event_center::communication::engine::indicator_engine::{CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, IndicatorEngineCommand};
use event_center::communication::engine::EngineResponse;
use event_center::communication::backtest_strategy::{GetIndicatorDataCmdPayload, GetIndicatorDataCommand, GetKlineDataCmdPayload, GetKlineDataCommand, InitIndicatorDataCmdPayload, InitIndicatorDataCommand, NodeResponse};
use event_center::EventCenterSingleton;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::oneshot;
use tokio::time::Duration;
use event_center::communication::Response;
use star_river_core::market::QuantData;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    selected_kline_key: KlineKey,                         // 回测K线缓存键
    indicator_keys: HashMap<IndicatorKey, (i32, String)>, // 指标缓存键 -> (配置id, 输出句柄id)
    kline_value: HashMap<IndicatorKey, Vec<Kline>>, // 指标缓存键 -> 指标值
    indicator_lookback: HashMap<IndicatorKey, usize>, // 指标缓存键 -> lookback
    min_interval_symbols: Vec<KlineKey>,
}

impl IndicatorNodeContext {
    pub fn new(
        base_context: BacktestBaseNodeContext,
        backtest_config: IndicatorNodeBacktestConfig,
        selected_kline_key: KlineKey,
        indicator_keys: HashMap<IndicatorKey, (i32, String)>,
    ) -> Self {
        Self {
            base_context,
            backtest_config,
            selected_kline_key,
            indicator_keys,
            kline_value: HashMap::new(),
            indicator_lookback: HashMap::new(),
            min_interval_symbols: vec![],
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols_ref(&self) -> &Vec<KlineKey> {
        &self.min_interval_symbols
    }

    pub fn get_indicator_keys_ref(&self) -> &HashMap<IndicatorKey, (i32, String)> {
        &self.indicator_keys
    }

    // 注册指标（初始化指标）向指标引擎发送注册请求
    // pub async fn register_indicator_cache_key(&self) -> Result<bool, String> {
    //     let mut is_all_success = true;
    //     // 遍历已配置的指标，注册指标缓存键
    //     for (indicator_key, _) in self.indicator_keys.iter() {
    //         let (resp_tx, resp_rx) = oneshot::channel();

    //         let register_indicator_params = AddKeyParams::new(
    //             self.get_strategy_id().clone(),
    //             indicator_key.clone().into(),
    //             None,
    //             Duration::from_secs(30),
    //             self.get_node_id().to_string(),
    //             resp_tx,
    //         );
    //         // self.get_command_publisher().send(register_indicator_command).await.unwrap();
    //         EventCenterSingleton::send_command(register_indicator_params.into())
    //             .await
    //             .unwrap();
    //         let response = resp_rx.await.unwrap();
    //         if !response.success() {
    //             is_all_success = false;
    //             break;
    //         }
    //     }

    //     Ok(is_all_success)
    // }

    // 获取已经计算好的回测指标数据
    async fn get_indicator_data(
        &self,
        indicator_key: &IndicatorKey,
        play_index: i32,
    ) -> Result<Indicator, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetIndicatorDataCmdPayload::new(
            indicator_key.clone(),
            Some(play_index),
            Some(1),
        );
        let get_indicator_cmd = GetIndicatorDataCommand::new(
            self.get_node_id().clone(),
            resp_tx,
            Some(payload),
        );
            
        self.get_strategy_command_sender().send(get_indicator_cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.indicator_series.last().unwrap().clone());
        }
        else {
            return Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id));
        }
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<bool, String> {
        let mut is_all_success = true;

        let kline_key = self.selected_kline_key.clone();
        let min_interval_symbols = self.get_min_interval_symbols_ref();

        // 如果当前IndicatorNode选择的kline_key不是最小周期交易对，则直接返回true
        if !min_interval_symbols.contains(&kline_key) {
            tracing::warn!("[{}] selected symbol is not min interval, skip", self.get_node_name());
            return Ok(true);
        }

        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();

        for (ind_key, _) in self.indicator_keys.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetKlineDataCmdPayload::new(
                kline_key.clone(),
                None,
                None,
            );

            // 获取所有K线
            let get_kline_series_cmd = GetKlineDataCommand::new(
                node_id.clone(),
                resp_tx,
                Some(payload),
            );
                
            self.get_strategy_command_sender().send(get_kline_series_cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.is_success() {

                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = CalculateHistoryIndicatorCmdPayload::new(
                    strategy_id.clone(),
                    node_id.clone(),
                    kline_key.clone().into(),
                    response.kline_series.clone(),
                    ind_key.indicator_config.clone(),
                );
                let cmd: IndicatorEngineCommand = CalculateHistoryIndicatorCommand::new(
                    node_id.clone(),
                    resp_tx,
                    Some(payload),
                ).into();
                
                EventCenterSingleton::send_command(cmd.into()).await.unwrap();
                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let payload = InitIndicatorDataCmdPayload::new(
                        ind_key.clone(),
                        response.indicators.clone(),
                    );
                    let update_indicator_params = InitIndicatorDataCommand::new(
                        node_id.clone(),
                        resp_tx,
                        Some(payload),
                    );
                    self.get_strategy_command_sender().send(update_indicator_params.into()).await.unwrap();
                    let response = resp_rx.await.unwrap();
                    if response.is_success() {
                        continue;
                    }
                } else {
                    is_all_success = false;
                    break;

                }
                
            }

            
        }
        Ok(is_all_success)
    }
}
