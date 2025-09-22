mod context_impl;
mod event_handler;

use super::indicator_node_type::IndicatorNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use event_center::communication::engine::cache_engine::CacheEngineResponse;
use event_center::communication::engine::cache_engine::{AddCacheKeyParams, GetCacheParams};
use event_center::communication::engine::indicator_engine::CalculateHistoryIndicatorParams;
use event_center::EventCenterSingleton;
use star_river_core::cache::key::{IndicatorKey, KlineKey};
use star_river_core::cache::CacheValue;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::time::Duration;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    pub is_registered: Arc<RwLock<bool>>,  // 是否已经注册指标
    selected_kline_key: KlineKey,               // 回测K线缓存键
    indicator_keys: HashMap<IndicatorKey,(i32, String)>, // 指标缓存键 -> (配置id, 输出句柄id)
    min_interval_symbols: Vec<KlineKey>,
}



impl IndicatorNodeContext {

    pub fn new(
        base_context: BacktestBaseNodeContext, 
        backtest_config: IndicatorNodeBacktestConfig, 
        selected_kline_key: KlineKey,
        indicator_keys: HashMap<IndicatorKey,(i32, String)>,
    ) -> Self {
        Self {
            base_context,
            backtest_config,
            is_registered: Arc::new(RwLock::new(false)),
            selected_kline_key,
            indicator_keys,
            min_interval_symbols: vec![],
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols_ref(&self) -> &Vec<KlineKey> {
        &self.min_interval_symbols
    }

    pub fn get_indicator_keys_ref(&self) -> &HashMap<IndicatorKey,(i32, String)> {
        &self.indicator_keys
    }


    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator_cache_key(&self) -> Result<bool, String> {
        let mut is_all_success = true;
        // 遍历已配置的指标，注册指标缓存键
        for (indicator_key, _) in self.indicator_keys.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();

            let register_indicator_params = AddCacheKeyParams::new(
                self.get_strategy_id().clone(),
                indicator_key.clone().into(),
                None,
                Duration::from_secs(30),
                self.get_node_id().to_string(),
                resp_tx,
            );
            // self.get_command_publisher().send(register_indicator_command).await.unwrap();
            EventCenterSingleton::send_command(register_indicator_params.into())
                .await
                .unwrap();
            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }
        }

        Ok(is_all_success)
    }

    // 获取已经计算好的回测指标数据
    async fn get_backtest_indicator_cache(
        &self,
        indicator_key: &IndicatorKey,
        play_index: i32,
    ) -> Result<Arc<CacheValue>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_params = GetCacheParams::new(
            self.base_context.strategy_id.clone(),
            self.base_context.node_id.clone(),
            indicator_key.clone().into(),
            Some(play_index as u32),
            Some(1),
            self.base_context.node_id.clone(),
            resp_tx,
        );
        // self.get_command_publisher().send(get_cache_command.into()).await.unwrap();
        EventCenterSingleton::send_command(get_cache_params.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        if get_cache_data_response.cache_data.len() == 1 {
                            return Ok(get_cache_data_response.cache_data[0].clone());
                        }
                    }
                    _ => {
                        return Err(format!(
                            "节点{}收到回测K线缓存数据失败",
                            self.base_context.node_id
                        ))
                    }
                }
            }
        }
        Err(format!(
            "节点{}收到回测K线缓存数据失败",
            self.base_context.node_id
        ))
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
        
        
        for (ind_key, _ ) in self.indicator_keys.iter() {
        
            let (resp_tx, resp_rx) = oneshot::channel();
            let calculate_backtest_indicator_params = CalculateHistoryIndicatorParams::new(
                strategy_id.clone(),
                node_id.clone(),
                kline_key.clone().into(),
                ind_key.indicator_config.clone(),
                node_id.clone(),
                resp_tx,
            );
            EventCenterSingleton::send_command(calculate_backtest_indicator_params.into())
                .await
                .unwrap();
            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }
}

