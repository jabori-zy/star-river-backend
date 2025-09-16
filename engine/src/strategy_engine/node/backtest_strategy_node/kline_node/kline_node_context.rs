mod context_impl;
mod event_handler;

use super::kline_node_type::KlineNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use event_center::communication::engine::cache_engine::{CacheEngineResponse, GetCacheParams};
use event_center::communication::engine::exchange_engine::RegisterExchangeParams;
use event_center::communication::engine::market_engine::GetKlineHistoryParams;
use event_center::communication::engine::EngineResponse;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload,
};
use event_center::EventCenterSingleton;
use heartbeat::Heartbeat;
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::CacheValue;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}


impl KlineNodeContext {
    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<EngineResponse, String> {
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();

        tracing::info!(
            node_id = %node_id,
            node_name = %node_name,
            account_id = %account_id,
            exchange = ?exchange,
            account_id = %account_id,
            "start to register exchange.");

        let (resp_tx, resp_rx) = oneshot::channel();
        let register_param = RegisterExchangeParams::new(account_id, exchange, node_id, resp_tx);

        EventCenterSingleton::send_command(register_param.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }

    // 从交易所获取k线历史
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to load backtest kline data from exchange");
        // 已配置的symbol
        let selected_symbols = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbols
            .clone();

        let mut is_all_success = true;
        // 遍历每一个symbol，从交易所获取k线历史
        for symbol in selected_symbols.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let get_kline_history_params = GetKlineHistoryParams::new(
                self.base_context.strategy_id.clone(),
                self.base_context.node_id.clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .selected_account
                    .account_id
                    .clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .selected_account
                    .exchange
                    .clone(),
                symbol.symbol.clone(),
                symbol.interval.clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .time_range
                    .clone(),
                self.base_context.node_id.clone(),
                resp_tx,
            );
            EventCenterSingleton::send_command(get_kline_history_params.into())
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

    // 从缓存引擎获取k线数据
    pub async fn get_history_kline_cache(
        &self,
        kline_key: &KlineKey,
        play_index: i32, // 缓存索引
    ) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_params = GetCacheParams::new(
            self.get_strategy_id().clone(),
            self.get_node_id().clone(),
            kline_key.clone().into(),
            Some(play_index as u32),
            Some(1),
            self.get_node_id().clone(),
            resp_tx,
        );

        EventCenterSingleton::send_command(get_cache_params.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        return Ok(get_cache_data_response.cache_data);
                    }
                    _ => {}
                }
            }
        }
        Err(format!("get history kline cache failed"))
    }

    fn get_kline_update_event(
        &self,
        handle_id: String,
        config_id: i32,
        kline_key: &KlineKey,
        index: i32, // 缓存索引
        kline_data: Vec<Arc<CacheValue>>,
    ) -> KlineNodeEvent {
        let payload = KlineUpdatePayload::new(config_id, index, kline_key.clone(), kline_data);
        KlineNodeEvent::KlineUpdate(
            KlineUpdateEvent::new(
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                handle_id,
                payload,
            )
            .into(),
        )
    }
}
