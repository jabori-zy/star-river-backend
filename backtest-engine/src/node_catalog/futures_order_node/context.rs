mod context_util;
mod event_handler;
mod node_handles;
mod order_handler;
mod status_handler;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::{
    custom_type::{InputHandleId, NodeId, NodeName, OrderId},
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
pub struct FuturesOrderNodeContext {
    metadata: FuturesOrderNodeMetadata,
    node_config: FuturesOrderNodeConfig,
    is_processing_order: Arc<RwLock<HashMap<InputHandleId, bool>>>, // 是否正在处理订单 input_handle_id -> is_processing_order
    database: DatabaseConnection,                                   // 数据库连接
    heartbeat: Arc<Mutex<Heartbeat>>,                               // 心跳
    // virtual_trading_system: Arc<Mutex<BacktestVts>>,                // 虚拟交易系统
    vts_command_sender: mpsc::Sender<VtsCommand>,
    pub vts_event_receiver: broadcast::Receiver<VtsEvent>,
    unfilled_virtual_order: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 未成交的虚拟订单列表 input_handle_id -> unfilled_virtual_order
    virtual_order_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 虚拟订单历史列表 input_handle_id -> virtual_order_history
    virtual_transaction_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualTransaction>>>>, // 虚拟交易明细历史列表 input_handle_id -> virtual_transaction_history
    symbol_info: Vec<Symbol>,                                                                  // 交易对信息
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
            unfilled_virtual_order: Arc::new(RwLock::new(HashMap::new())),
            virtual_order_history: Arc::new(RwLock::new(HashMap::new())),
            virtual_transaction_history: Arc::new(RwLock::new(HashMap::new())),
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
        // let virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        // let order = virtual_trading_system_guard.get_order_by_id(&order_id).unwrap();
        // Ok(order.order_status.clone())
        Ok(OrderStatus::Created)
    }
}
