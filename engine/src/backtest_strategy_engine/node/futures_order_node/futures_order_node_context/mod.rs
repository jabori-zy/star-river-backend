mod context_impl;
mod event_handler;
mod order_handler;
mod status_handler;

use super::futures_order_node_types::*;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use event_center::EventCenterSingleton;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::GetStrategyKeysCommand;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::InputHandleId;
use star_river_core::custom_type::OrderId;
use star_river_core::key::Key;
use star_river_core::order::OrderStatus;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use star_river_core::virtual_trading_system::event::VirtualTradingSystemEventReceiver;
use star_river_core::error::engine_error::strategy_engine_error::node_error::futures_order_node_error::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use virtual_trading::VirtualTradingSystem;
use event_center::communication::engine::market_engine::{GetSymbolInfoCmdPayload, GetSymbolInfoCommand, MarketEngineCommand};
use star_river_core::market::Symbol;

#[derive(Debug)]
pub struct FuturesOrderNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub node_config: FuturesOrderNodeBacktestConfig,
    pub is_processing_order: Arc<RwLock<HashMap<InputHandleId, bool>>>, // 是否正在处理订单 input_handle_id -> is_processing_order
    pub database: DatabaseConnection,                                   // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>,                               // 心跳
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,       // 虚拟交易系统
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver, // 虚拟交易系统事件接收器
    pub unfilled_virtual_order: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 未成交的虚拟订单列表 input_handle_id -> unfilled_virtual_order
    pub virtual_order_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 虚拟订单历史列表 input_handle_id -> virtual_order_history
    pub virtual_transaction_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualTransaction>>>>, // 虚拟交易明细历史列表 input_handle_id -> virtual_transaction_history
    symbol_info: Vec<Symbol>, // 交易对信息
}

impl Clone for FuturesOrderNodeContext {
    fn clone(&self) -> Self {
        Self {
            base_context: self.base_context.clone(),
            node_config: self.node_config.clone(),
            is_processing_order: self.is_processing_order.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            virtual_trading_system_event_receiver: self.virtual_trading_system_event_receiver.resubscribe(),
            unfilled_virtual_order: self.unfilled_virtual_order.clone(),
            virtual_order_history: self.virtual_order_history.clone(),
            virtual_transaction_history: self.virtual_transaction_history.clone(),
            symbol_info: self.symbol_info.clone(),
        }
    }
}

impl FuturesOrderNodeContext {


    pub fn new(
        base_context: BacktestBaseNodeContext, 
        node_config: FuturesOrderNodeBacktestConfig, 
        database: DatabaseConnection, 
        heartbeat: Arc<Mutex<Heartbeat>>, 
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, 
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver
    ) -> Self {
        Self {
            base_context,
            node_config,
            database,
            heartbeat,
            virtual_trading_system,
            virtual_trading_system_event_receiver,
            is_processing_order: Arc::new(RwLock::new(HashMap::new())),
            unfilled_virtual_order: Arc::new(RwLock::new(HashMap::new())),
            virtual_order_history: Arc::new(RwLock::new(HashMap::new())),
            virtual_transaction_history: Arc::new(RwLock::new(HashMap::new())),
            symbol_info: vec![],
        }
    }




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
        virtual_order_history_guard.entry(input_handle_id.to_string()).and_modify(|orders| {
            orders.retain(|order| order.order_id != virtual_order_id);
        });
    }

    async fn add_virtual_transaction_history(&mut self, input_handle_id: &InputHandleId, virtual_transaction: VirtualTransaction) {
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

        self.get_strategy_command_sender().send(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.keys.clone());
        } else {
            return Err("获取策略缓存键失败".to_string());
        }
    }
}
