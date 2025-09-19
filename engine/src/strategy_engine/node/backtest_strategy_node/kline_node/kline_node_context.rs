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
use star_river_core::cache::Key;
use std::collections::HashMap;
use star_river_core::market::KlineInterval;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    strategy_keys: Vec<Key>,
    is_min_interval_symbol: HashMap<Key, bool>,
}


impl KlineNodeContext {

    pub fn new(base_context: BacktestBaseNodeContext, backtest_config: KlineNodeBacktestConfig, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        Self {
            base_context,
            exchange_is_registered: Arc::new(RwLock::new(false)),
            data_is_loaded: Arc::new(RwLock::new(false)),
            backtest_config,
            heartbeat,
            strategy_keys: vec![],
            is_min_interval_symbol: HashMap::new(),
        }
    }


    pub fn set_strategy_keys(&mut self, strategy_keys: Vec<Key>) {
        self.strategy_keys = strategy_keys;
    }

    pub fn get_strategy_keys_ref(&self) -> &Vec<Key> {
        &self.strategy_keys
    }

    pub fn init_is_min_interval_symbol(&mut self) {

        // 过滤出klineKey中，每个symbol的interval最小的klineKey
        let mut symbol_groups: HashMap<String, Vec<&Key>> = HashMap::new();

        // tracing::debug!("strategy_keys: {:#?}", self.strategy_keys.iter().map(|key| (key.get_symbol(), key.get_interval())).collect::<Vec<(String, KlineInterval)>>());
        // 按symbol分组
        for key in self.strategy_keys.iter() {
            if matches!(key, Key::Kline(_)) {
                let symbol = key.get_symbol();
                symbol_groups.entry(symbol).or_insert_with(Vec::new).push(key);
            }
        }

        // 获取每个symbol组内interval最小的key
        let min_interval_symbol: Vec<&Key> = symbol_groups
            .values()
            .filter_map(|keys| {
                keys.iter().min_by_key(|key| key.get_interval()).copied()
            })
            .collect();
        // tracing::debug!("min_interval_symbol: {:#?}", min_interval_symbol.iter().map(|key| (key.get_symbol(), key.get_interval())).collect::<Vec<(String, KlineInterval)>>());
        
        let selected_symbols = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbols.clone();
        let exchange = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
        let time_range = self.backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();
        selected_symbols.iter().for_each(|symbol| {
            let key: Key = KlineKey::new(exchange.clone(), symbol.symbol.clone(), symbol.interval.clone(), Some(time_range.start_date.to_string()), Some(time_range.end_date.to_string())).into();

            // 检查当前key是否在最小interval列表中
            let is_min_interval = min_interval_symbol.iter().any(|min_interval_key| *min_interval_key == &key);
            self.is_min_interval_symbol.insert(key, is_min_interval);
        });
        // tracing::debug!("is_min_interval_symbol: {:#?}", self.is_min_interval_symbol.iter().map(|(key, is_min_interval)| (key.get_symbol(), key.get_interval(), is_min_interval.clone())).collect::<Vec<(String, KlineInterval, bool)>>());


    }


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

    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!("[{}] start to load backtest kline data from exchange", self.base_context.node_name);


        // 已配置的symbol
        // let selected_symbols = self
        //     .backtest_config
        //     .exchange_mode_config
        //     .as_ref()
        //     .unwrap()
        //     .selected_symbols
        //     .clone();

        let mut is_all_success = true;

        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();
        let account_id = self.backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self.backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let time_range = self.backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .time_range
            .clone();

        // 遍历每一个symbol，从交易所获取k线历史
        for (symbol, is_min_interval) in self.is_min_interval_symbol.iter() {
            // 如果is_min_interval为false，则跳过
            if !is_min_interval {
                tracing::warn!("[{}] symbol: {}-{}, is not min interval, skip", self.get_node_name(), symbol.get_symbol(), symbol.get_interval());
                continue;
            }
            let (resp_tx, resp_rx) = oneshot::channel();
            let get_kline_history_params = GetKlineHistoryParams::new(
                strategy_id,
                node_id.clone(),
                account_id.clone(),
                exchange.clone(),
                symbol.get_symbol().clone(),
                symbol.get_interval().clone(),
                time_range.clone(),
                node_id.clone(),
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
