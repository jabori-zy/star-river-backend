mod command_handler;
mod context_impl;
mod event_handler;
mod utils;

use super::kline_node_type::KlineNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use event_center::communication::backtest_strategy::{GetKlineDataCmdPayload, GetKlineDataCommand, InitKlineDataCmdPayload, InitKlineDataCommand, NodeResponse};
use event_center::communication::engine::exchange_engine::{ExchangeEngineCommand, RegisterExchangeCmdPayload, RegisterExchangeCommand, RegisterExchangeRespPayload};
use event_center::communication::engine::market_engine::{GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand};
use event_center::communication::Response;
use event_center::EventCenterSingleton;
use event_center::communication::engine::EngineResponse;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload,
};
use heartbeat::Heartbeat;
use star_river_core::cache::KeyTrait;
use star_river_core::cache::key::KlineKey;
use star_river_core::custom_type::PlayIndex;
use star_river_core::error::engine_error::node_error::backtest_strategy_node_error::kline_node_error::*;
use star_river_core::market::Kline;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    min_interval_symbols: Vec<KlineKey>,
    selected_symbol_keys: HashMap<KlineKey, (i32, String)>, // 已配置的symbol键 -> (配置id, 输出句柄id)
}

impl KlineNodeContext {
    pub fn new(
        base_context: BacktestBaseNodeContext,
        backtest_config: KlineNodeBacktestConfig,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let exchange = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let time_range = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .time_range
            .clone();

        let selected_symbol_keys = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbols
            .iter()
            .map(|symbol| {
                let kline_key = KlineKey::new(
                    exchange.clone(),
                    symbol.symbol.clone(),
                    symbol.interval.clone(),
                    Some(time_range.start_date.to_string()),
                    Some(time_range.end_date.to_string()),
                );
                (kline_key, (symbol.config_id, symbol.output_handle_id.clone()))
            })
            .collect();

        Self {
            base_context,
            exchange_is_registered: Arc::new(RwLock::new(false)),
            data_is_loaded: Arc::new(RwLock::new(false)),
            backtest_config,
            heartbeat,
            min_interval_symbols: vec![],
            selected_symbol_keys,
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols_ref(&self) -> &Vec<KlineKey> {
        &self.min_interval_symbols
    }

    pub fn get_selected_symbol_keys_ref(&self) -> &HashMap<KlineKey, (i32, String)> {
        &self.selected_symbol_keys
    }

    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<EngineResponse<RegisterExchangeRespPayload>, String> {
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

        tracing::info!("[{}] start to register exchange [{}]", node_name, exchange);

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = RegisterExchangeCmdPayload::new(
            account_id, 
            exchange
        );
        let cmd: ExchangeEngineCommand = RegisterExchangeCommand::new(node_id, resp_tx, Some(payload)).into();

        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }

    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!(
            "[{}] start to load backtest kline data from exchange",
            self.base_context.node_name
        );

        let mut is_all_success = true;

        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();

        // 遍历每一个symbol，从交易所获取k线历史
        for (symbol_key, _) in self.selected_symbol_keys.iter() {
            // 如果key不在最小周期交易对列表中，则跳过
            if !self.min_interval_symbols.contains(&symbol_key) {
                tracing::warn!(
                    "[{}] symbol: {}-{}, is not min interval, skip",
                    self.get_node_name(),
                    symbol_key.get_symbol(),
                    symbol_key.get_interval()
                );
                continue;
            }
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetKlineHistoryCmdPayload::new(
                strategy_id,
                node_id.clone(),
                account_id.clone(),
                symbol_key.get_exchange(),
                symbol_key.get_symbol(),
                symbol_key.get_interval(),
                symbol_key.get_time_range().unwrap(),
            );
            let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(),resp_tx,Some(payload)).into();
            EventCenterSingleton::send_command(cmd.into())
                .await
                .unwrap();

            let response = resp_rx.await.unwrap();
            if response.is_success() {
                let kline_history = response.kline_history.clone();
                tracing::debug!(
                    "[{}] get kline history from exchange success, symbol: {}-{}, kline history length: {:#?}",
                    self.get_node_name(),
                    symbol_key.get_symbol(),
                    symbol_key.get_interval(),
                    kline_history.len()
                );

                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = InitKlineDataCmdPayload::new(
                    symbol_key.clone(),
                    kline_history,
                );
                
                let init_kline_data_command= InitKlineDataCommand::new(
                    self.get_node_id().clone(),
                    resp_tx,
                    Some(payload),
                );

                self.get_strategy_command_sender()
                    .send(init_kline_data_command.into())
                    .await
                    .unwrap();

                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    continue;
                }
                
            }

            else {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }

    // 从策略中获取k线数据
    pub async fn get_kline(
        &self,
        kline_key: &KlineKey,
        play_index: PlayIndex, // 播放索引
    ) -> Result<Vec<Kline>, KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(
            kline_key.clone(),
            Some(play_index),
            Some(1),
        );
        let get_kline_params = GetKlineDataCommand::new(
            self.get_node_id().clone(), 
            resp_tx,
            Some(payload),
        );
        
        self.get_strategy_command_sender().send(get_kline_params.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.kline_series.clone());
        }
        Err(GetKlineDataSnafu {
            node_name: self.get_node_name().clone(),
            kline_key: kline_key.get_key_str(),
            play_index: play_index as u32,
        }
        .fail()?)
    }

    fn get_kline_update_event(
        &self,
        handle_id: String,
        config_id: i32,
        should_calculate: bool,
        kline_key: &KlineKey,
        index: i32, // 缓存索引
        kline_data: Kline,
    ) -> KlineNodeEvent {
        let payload = KlineUpdatePayload::new(config_id, index, should_calculate, kline_key.clone(), kline_data);
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
