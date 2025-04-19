use exchange_client::ExchangeClient;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use sea_orm::DatabaseConnection;
use crate::EngineName;
use async_trait::async_trait;
use crate::EngineContext;
use std::any::Any;
use event_center::order_event::OrderEvent;
use types::order::Order;
use crate::exchange_engine::ExchangeEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use types::market::Exchange;
use crate::Engine;
use event_center::command_event::order_engine_command::GetTransactionDetailParams;
use database::mutation::transaction_detail_mutation::TransactionDetailMutation;

#[derive(Debug)]
pub struct TransactionEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub database: DatabaseConnection,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
}

impl Clone for TransactionEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            database: self.database.clone(),
            exchange_engine: self.exchange_engine.clone(),
        }
    }
}

#[async_trait]
impl EngineContext for TransactionEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Order(order_event) => {
                match order_event {
                    OrderEvent::OrderFilled(order) => {
                        self.get_transaction_detail(order).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    
}


impl TransactionEngineContext {

    async fn get_exchange(&self, account_id: &i32) -> Result<Box<dyn ExchangeClient>, String> {
        // 1. 先检查交易所注册状态
        let is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(account_id).await
        };

        if !is_registered {
            return Err("交易所未注册".to_string());
        }

        // 2. 获取上下文（新的锁范围）
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };

        // 3. 获取读锁
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read
            .as_any()
            .downcast_ref::<ExchangeEngineContext>()
            .unwrap();

        exchange_engine_context_guard.get_exchange(account_id).await
    }

    async fn get_transaction_detail(&mut self, order: Order) -> Result<(), String> {
        // 订单已完成，获取交易明细
        tracing::info!("订单已完成，获取交易明细: {:?}", order);
        // 获取交易明细
        tracing::info!("订单已成交, 获取持仓: {:?}", order);

        let exchange = self.get_exchange(&order.account_id).await.unwrap();
        // 根据order_id获取交易明细
        let get_transaction_detail_params = GetTransactionDetailParams {
            strategy_id: order.strategy_id.clone(),
            node_id: order.node_id.clone(),
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            transaction_id: None,
            position_id: None,
            order_id: Some(order.exchange_order_id),
        };
        let transaction_detail = exchange.get_transaction_detail(get_transaction_detail_params).await.unwrap();
        // 入库
        let transaction_detail = TransactionDetailMutation::insert_transaction_detail(
            &self.database,
            order.strategy_id,
            order.node_id,
            transaction_detail,
        ).await.unwrap();
        tracing::info!("入库交易明细成功: {:?}", transaction_detail);

        Ok(())
    }
}


