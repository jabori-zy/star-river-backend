mod context_impl;
mod event_handler;
mod order_handler;

use super::futures_order_node_types::*;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use event_center::communication::backtest_strategy::GetStrategyKeysCommand;
use event_center::communication::engine::cache_engine::CacheEngineResponse;
use event_center::communication::engine::cache_engine::GetCacheParams;
use event_center::communication::engine::EngineResponse;
use event_center::communication::backtest_strategy::NodeResetResponse;
use event_center::communication::Response;
use event_center::EventCenterSingleton;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::CacheValue;
use star_river_core::cache::Key;
use star_river_core::custom_type::InputHandleId;
use star_river_core::custom_type::OrderId;
use star_river_core::market::KlineInterval;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::order::OrderStatus;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use star_river_core::virtual_trading_system::event::VirtualTradingSystemEventReceiver;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use virtual_trading::VirtualTradingSystem;

#[derive(Debug)]
pub struct FuturesOrderNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: FuturesOrderNodeBacktestConfig,
    pub is_processing_order: Arc<RwLock<HashMap<InputHandleId, bool>>>, // 是否正在处理订单 input_handle_id -> is_processing_order
    pub database: DatabaseConnection,                                   // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>,                               // 心跳
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,       // 虚拟交易系统
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver, // 虚拟交易系统事件接收器
    pub unfilled_virtual_order: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 未成交的虚拟订单列表 input_handle_id -> unfilled_virtual_order
    pub virtual_order_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 虚拟订单历史列表 input_handle_id -> virtual_order_history
    pub virtual_transaction_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualTransaction>>>>, // 虚拟交易明细历史列表 input_handle_id -> virtual_transaction_history
    pub min_kline_interval: Option<KlineInterval>, // 最小K线间隔(最新价格只需要获取最小间隔的价格即可)
}

impl Clone for FuturesOrderNodeContext {
    fn clone(&self) -> Self {
        Self {
            base_context: self.base_context.clone(),
            backtest_config: self.backtest_config.clone(),
            is_processing_order: self.is_processing_order.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            virtual_trading_system_event_receiver: self.virtual_trading_system_event_receiver.resubscribe(),
            unfilled_virtual_order: self.unfilled_virtual_order.clone(),
            virtual_order_history: self.virtual_order_history.clone(),
            virtual_transaction_history: self.virtual_transaction_history.clone(),
            min_kline_interval: self.min_kline_interval.clone(),
        }
    }
}

impl FuturesOrderNodeContext {
    async fn set_is_processing_order(&mut self, input_handle_id: &InputHandleId, is_processing_order: bool) {
        self.is_processing_order
            .write()
            .await
            .insert(input_handle_id.to_string(), is_processing_order);
    }

    // 添加未成交的虚拟订单
    async fn add_unfilled_virtual_order(&mut self, input_handle_id: &InputHandleId, virtual_order: VirtualOrder) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        // tracing::info!("{}: 订单已添加到未成交的虚拟订单列表: {:?}", self.get_node_id(), virtual_order);
        unfilled_virtual_order_guard
            .entry(input_handle_id.to_string())
            .or_insert(vec![])
            .push(virtual_order);
    }

    async fn remove_unfilled_virtual_order(&mut self, input_handle_id: &InputHandleId, virtual_order_id: OrderId) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        unfilled_virtual_order_guard
            .entry(input_handle_id.to_string())
            .and_modify(|orders| {
                orders.retain(|order| order.order_id != virtual_order_id);
            });
    }

    async fn add_virtual_order_history(&mut self, input_handle_id: &InputHandleId, virtual_order: VirtualOrder) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard
            .entry(input_handle_id.to_string())
            .or_insert(vec![])
            .push(virtual_order);
    }

    async fn remove_virtual_order_history(&mut self, input_handle_id: &InputHandleId, virtual_order_id: OrderId) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard
            .entry(input_handle_id.to_string())
            .and_modify(|orders| {
                orders.retain(|order| order.order_id != virtual_order_id);
            });
    }

    async fn add_virtual_transaction_history(
        &mut self,
        input_handle_id: &InputHandleId,
        virtual_transaction: VirtualTransaction,
    ) {
        let mut virtual_transaction_history_guard = self.virtual_transaction_history.write().await;
        virtual_transaction_history_guard
            .entry(input_handle_id.to_string())
            .or_insert(vec![])
            .push(virtual_transaction);
    }

    // 判断是否可以创建订单
    async fn can_create_order(&mut self, input_handle_id: &InputHandleId) -> bool {
        let is_processing_order_guard = self.is_processing_order.read().await;
        let is_processing_order = *is_processing_order_guard.get(input_handle_id).unwrap_or(&false);

        let unfilled_virtual_order_guard = self.unfilled_virtual_order.read().await;
        let unfilled_virtual_order_len = unfilled_virtual_order_guard.get(input_handle_id).map_or(0, |v| v.len());

        !(is_processing_order || unfilled_virtual_order_len > 0)
    }

    async fn check_order_status(&mut self, order_id: OrderId) -> Result<OrderStatus, String> {
        let virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let order = virtual_trading_system_guard.get_order(order_id);
        if let Some(order) = order {
            return Ok(order.order_status.clone());
        }
        Err("订单不存在".to_string())
    }

    pub async fn monitor_unfilled_order(&mut self) {
        let unfilled_virtual_order = self.unfilled_virtual_order.clone();
        let is_processing_order = self.is_processing_order.clone();
        let database = self.database.clone();
        let node_name = self.get_node_name().clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat
            .register_async_task(
                format!("{}监控未成交订单", self.get_node_name()),
                move || {
                    let unfilled_virtual_order = unfilled_virtual_order.clone();
                    let is_processing_order = is_processing_order.clone();
                    let database = database.clone();
                    let node_name = node_name.clone();
                    async move {
                        Self::process_unfilled_virtual_order(
                            node_name,
                            unfilled_virtual_order,
                            is_processing_order,
                            database,
                        )
                        .await
                    }
                },
                10,
            )
            .await;
    }

    async fn process_unfilled_virtual_order(
        node_name: String,
        unfilled_virtual_order: Arc<RwLock<HashMap<String, Vec<VirtualOrder>>>>,
        is_processing_order: Arc<RwLock<HashMap<String, bool>>>,
        database: DatabaseConnection,
    ) {
        let unfilled_virtual_order_clone = {
            let unfilled_order_guard = unfilled_virtual_order.read().await;
            unfilled_order_guard.clone()
        };
        // 如果当前没有正在处理的订单，则直接返回
        if unfilled_virtual_order_clone.len() == 0 {
            // tracing::info!("{}: 没有未成交订单", node_name);
            return;
        }
        // 如果当前有正在处理的订单，则获取订单的最新状态
    }

    async fn get_strategy_keys(&mut self) -> Result<Vec<Key>, String> {
        let (tx, rx) = oneshot::channel();
        
        let cmd = GetStrategyKeysCommand::new(self.get_node_id().clone(), tx, None);

        self.get_strategy_command_sender()
            .send(cmd.into())
            .await
            .unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.keys.clone());
        } else {
            return Err("获取策略缓存键失败".to_string());
        }
    }

    // 获取K线缓存数据
    // 获取interval最小的K线缓存数据
    async fn get_kline_cache_data(&mut self) -> Result<Vec<Arc<CacheValue>>, String> {
        // 如果min_kline_interval为None，则获取策略的缓存键
        if self.min_kline_interval.is_none() {
            let cache_keys = self.get_strategy_keys().await;
            // 获取成功
            if let Ok(cache_keys) = cache_keys {
                // 过滤出K线缓存key
                let kline_keys = cache_keys
                    .iter()
                    .filter(|k| matches!(k, Key::Kline(_)))
                    .collect::<Vec<&Key>>();
                // 获取interval最小的K线缓存数据
                // 如果列表长度为1，则唯一的key就是最小interval的key
                if kline_keys.len() == 1 {
                    self.min_kline_interval = Some(kline_keys[0].get_interval());
                } else if !kline_keys.is_empty() {
                    // 如果列表长度大于1，则需要根据interval排序，获取最小的interval的key
                    let min_interval_key = kline_keys.iter().min_by_key(|k| k.get_interval()).unwrap(); // 这里可以安全unwrap，因为我们已经检查了不为空
                    self.min_kline_interval = Some(min_interval_key.get_interval());
                }
            }
        }
        // 如果min_kline_interval不为None，则获取K线缓存数据
        if let Some(min_kline_interval) = &self.min_kline_interval {
            let cache_key = KlineKey::new(
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .selected_account
                    .exchange
                    .clone(),
                self.backtest_config.futures_order_configs[0].symbol.clone(),
                min_kline_interval.clone(),
                Some(
                    self.backtest_config
                        .exchange_mode_config
                        .as_ref()
                        .unwrap()
                        .time_range
                        .start_date
                        .to_string(),
                ),
                Some(
                    self.backtest_config
                        .exchange_mode_config
                        .as_ref()
                        .unwrap()
                        .time_range
                        .end_date
                        .to_string(),
                ),
            );

            let play_index = self.get_play_index() as u32;

            let (tx, rx) = oneshot::channel();
            let get_cache_params = GetCacheParams::new(
                self.base_context.strategy_id.clone(),
                self.base_context.node_id.clone(),
                cache_key.clone().into(),
                Some(play_index),
                Some(1),
                self.base_context.node_id.clone(),
                tx,
            );

            // self.get_command_publisher().send(get_cache_command.into()).await.unwrap();
            EventCenterSingleton::send_command(get_cache_params.into())
                .await
                .unwrap();

            let reponse = rx.await.unwrap();
            match reponse {
                EngineResponse::CacheEngine(CacheEngineResponse::GetCacheData(get_cache_data_response)) => {
                    // tracing::info!("{}: 获取K线缓存数据成功: {:?}", self.get_node_id(), get_cache_data_response.cache_data);
                    return Ok(get_cache_data_response.cache_data);
                }
                _ => return Err("获取K线缓存数据失败".to_string()),
            }
        } else {
            return Err("获取K线缓存数据失败".to_string());
        }
    }
}
