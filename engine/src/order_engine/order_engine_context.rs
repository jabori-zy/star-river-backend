use tokio::sync::broadcast;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use std::sync::Arc;
use event_center::command_event::CommandEvent;
use event_center::command_event::order_engine_command::OrderEngineCommand;
use event_center::command_event::order_engine_command::CreateOrderParams;
use event_center::EventPublisher;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use std::collections::HashMap;
use types::order::Order;
use sea_orm::DatabaseConnection;
use database::mutation::order_mutation::OrderMutation;



#[derive(Debug)]
pub struct OrderEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub unfilled_orders: HashMap<i32, Vec<Order>>,
    pub database: DatabaseConnection,
}


impl Clone for OrderEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            exchange_engine: self.exchange_engine.clone(),
            unfilled_orders: self.unfilled_orders.clone(),
            database: self.database.clone(),

        }
    }
}


#[async_trait]
impl EngineContext for OrderEngineContext {

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
            Event::Command(command_event) => {
                match command_event {
                    CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(params)) => {
                        self.create_order(params).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }

}


impl OrderEngineContext {
    async fn create_order(&mut self, params: CreateOrderParams) -> Result<(), String> {
        tracing::info!("订单引擎收到创建订单命令: {:?}", params);

        // 1. 先检查注册状态
        let is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&params.order_request.exchange).await
        };

        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", &params.order_request.exchange));
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

        let exchange = exchange_engine_context_guard.get_exchange_ref(&params.order_request.exchange).await.unwrap();


        let order_request = params.order_request;
        let order = exchange.create_order(order_request).await.unwrap();

        // 入库
        if OrderMutation::create_order(&self.database, order.clone()).await.is_err() {
            tracing::error!("订单入库失败: {:?}", order);
            return Err("订单入库失败".to_string());
        } else {
            tracing::info!("订单入库成功: {:?}", order);
        }
        Ok(())
    }

    

}