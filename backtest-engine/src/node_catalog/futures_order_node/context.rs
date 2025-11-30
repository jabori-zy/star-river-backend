mod context_util;
mod event_handler;
mod node_handles;
mod order_handler;
mod status_handler;
mod config_filter;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::{
    custom_type::{NodeId, NodeName, OrderId},
    instrument::Symbol,
    order::OrderStatus,
};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use virtual_trading::{
    command::VtsCommand,
    event::VtsEvent,
    types::{order::VirtualOrder, transaction::VirtualTransaction},
};

use super::{futures_order_node_types::FuturesOrderNodeConfig, state_machine::FuturesOrderNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::FuturesOrderNodeError, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
};

pub type FuturesOrderNodeMetadata =
    NodeMetadata<FuturesOrderNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FuturesOrderNodeContext {
    metadata: FuturesOrderNodeMetadata,
    node_config: FuturesOrderNodeConfig,
    is_processing_order: Arc<RwLock<HashMap<i32, (bool, i32)>>>, // 是否正在处理订单 order_config_id -> (is_processing_order, warn_log_send_count)
    database: DatabaseConnection,                                // 数据库连接
    heartbeat: Arc<Mutex<Heartbeat>>,                            // 心跳
    vts_command_sender: mpsc::Sender<VtsCommand>,
    pub(crate) vts_event_receiver: broadcast::Receiver<VtsEvent>,
    unfilled_virtual_order: Arc<RwLock<Vec<VirtualOrder>>>, // 未成交的虚拟订单列表
    virtual_order_history: Arc<RwLock<Vec<VirtualOrder>>>,  // 虚拟订单历史列表
    virtual_transaction_history: Arc<RwLock<Vec<VirtualTransaction>>>, // 虚拟交易明细历史列表
    symbol_info: Vec<Symbol>,                               // 交易对信息
}

impl FuturesOrderNodeContext {
    pub fn new(
        metadata: FuturesOrderNodeMetadata,
        node_config: FuturesOrderNodeConfig,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        vts_command_sender: mpsc::Sender<VtsCommand>,
        vts_event_receiver: broadcast::Receiver<VtsEvent>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            is_processing_order: Arc::new(RwLock::new(HashMap::new())),
            database,
            heartbeat,
            vts_command_sender,
            vts_event_receiver,
            unfilled_virtual_order: Arc::new(RwLock::new(Vec::new())),
            virtual_order_history: Arc::new(RwLock::new(Vec::new())),
            virtual_transaction_history: Arc::new(RwLock::new(Vec::new())),
            symbol_info: vec![],
        }
    }
}

impl FuturesOrderNodeContext {
    pub fn node_config(&self) -> &FuturesOrderNodeConfig {
        &self.node_config
    }
}

impl NodeMetaDataExt for FuturesOrderNodeContext {
    type StateMachine = FuturesOrderNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;
    type Error = FuturesOrderNodeError;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for FuturesOrderNodeContext {
    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error> {
        crate::node::node_utils::NodeUtils::mount_node_cycle_tracker(node_id, node_name, cycle_tracker, self.strategy_command_sender())
            .await?;
        Ok(())
    }
}

impl FuturesOrderNodeContext {
    async fn set_is_processing_order(&mut self, order_config_id: i32, is_processing_order: bool) {
        self.is_processing_order
            .write()
            .await
            .insert(order_config_id, (is_processing_order, 0));
    }

    async fn increment_warn_log_send_count(&mut self, order_config_id: i32) {
        self.is_processing_order
            .write()
            .await
            .entry(order_config_id)
            .and_modify(|(_, count)| *count += 1);
    }

    async fn warn_log_send_count(&mut self, order_config_id: &i32) -> i32 {
        tracing::debug!("warn_log_send_count: {:?}", self.is_processing_order.read().await);
        self.is_processing_order.read().await.get(order_config_id).unwrap_or(&(false, 0)).1
    }

    // 添加未成交的虚拟订单
    async fn add_unfilled_virtual_order(&mut self, virtual_order: VirtualOrder) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        // tracing::info!("{}: 订单已添加到未成交的虚拟订单列表: {:?}", self.get_node_id(), virtual_order);
        unfilled_virtual_order_guard.push(virtual_order);
    }

    async fn remove_unfilled_virtual_order(&mut self, virtual_order_id: OrderId) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        unfilled_virtual_order_guard.retain(|order| order.order_id != virtual_order_id);
    }

    async fn add_virtual_order_history(&mut self, virtual_order: VirtualOrder) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard.push(virtual_order);
    }

    async fn remove_virtual_order_history(&mut self, virtual_order_id: OrderId) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard.retain(|order| order.order_id != virtual_order_id);
    }

    async fn add_virtual_transaction_history(&mut self, virtual_transaction: VirtualTransaction) {
        let mut virtual_transaction_history_guard = self.virtual_transaction_history.write().await;
        virtual_transaction_history_guard.push(virtual_transaction);
    }

    // 判断是否可以创建订单
    async fn can_create_order(&mut self, order_config_id: &i32) -> bool {
        let is_processing_order_guard = self.is_processing_order.read().await;
        let is_processing_order = is_processing_order_guard.get(order_config_id).unwrap_or(&(false, 0)).0;

        // let unfilled_virtual_order_guard = self.unfilled_virtual_order.read().await;
        // let have_unfilled_order = unfilled_virtual_order_guard.iter().any(|order| order.order_config_id == *order_config_id);

        !is_processing_order
    }

    async fn check_order_status(&mut self, order_id: OrderId) -> Result<OrderStatus, String> {
        // let virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        // let order = virtual_trading_system_guard.get_order_by_id(&order_id).unwrap();
        // Ok(order.order_status.clone())
        Ok(OrderStatus::Created)
    }
}
