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
use types::order::{OriginalOrder, Order};
use sea_orm::DatabaseConnection;
use database::mutation::order_mutation::OrderMutation;
use tokio::sync::RwLock;
use heartbeat::Heartbeat;
use types::order::OrderStatus;
use event_center::order_event::OrderEvent;
use exchange_client::ExchangeClient;

#[derive(Debug)]
pub struct OrderEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub unfilled_orders: Arc<RwLock<HashMap<i64, Vec<Order>>>>, // 未成交订单, 需要持续监控, 策略id作为key
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>
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
            heartbeat: self.heartbeat.clone(),

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
            exchange_engine_guard.is_registered(&params.account_id).await
        };

        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", &params.account_id));
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

        let exchange = exchange_engine_context_guard.get_exchange_ref(&params.account_id).await.unwrap();

        let exchange_order = exchange.create_order(params.clone()).await.unwrap();

        tracing::info!("创建订单成功: {:?}", exchange_order);
        // 先入库，分配系统的订单id
        if let Ok(order) = OrderMutation::insert_order(&self.database, params.base_params.strategy_id.clone(), params.base_params.node_id.clone(), params.account_id, exchange_order.clone()).await {
            tracing::info!("订单入库成功: {:?}", order);
            // 如果订单状态为已成交，则通知持仓引擎，订单已成交
            if exchange_order.get_order_status() == OrderStatus::Filled {
                // 通知持仓引擎，订单已成交
                let order_event = OrderEvent::OrderFilled(order.clone());
                self.event_publisher.publish(order_event.into()).unwrap();

            } 
            // 如果订单状态为其他的状态，则将订单添加到未成交订单列表
            else {
                // 将订单添加到未成交订单列表
                let mut unfilled_orders = self.unfilled_orders.write().await;
                unfilled_orders.entry(params.base_params.strategy_id).or_insert(vec![]).push(order);
            }
        } else {
            tracing::error!("订单入库失败: {:?}", exchange_order);
            return Err("订单入库失败".to_string());
        }

        
        Ok(())
    }



    async fn process_unfilled_orders(
        unfilled_orders: Arc<RwLock<HashMap<i64, Vec<Order>>>>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        event_publisher: EventPublisher,
        database: DatabaseConnection,
    ) {
        // tracing::debug!("执行监控未成交订单");
        let unfilled_orders_clone = {
            let unfilled_orders = unfilled_orders.read().await;
            unfilled_orders.clone()
        };

        // 如果hashmap为空，则直接返回
        if unfilled_orders_clone.is_empty() {
            // tracing::debug!("未成交订单列表为空");
            return;
        }

        // 遍历未成交订单
        for (strategy_id, orders) in unfilled_orders_clone.iter() {
            // 先判断列表的长度
            // 如果策略的订单列表为空, 则跳过
            if orders.len() == 0 {
                // tracing::debug!("策略 {:?} 的订单列表为空, 跳过", strategy_id);
                continue;
            }
            // 列表不为空, 则遍历列表
            for order in orders {
                let exchange_engine_guard = exchange_engine.lock().await;
                let exchange = exchange_engine_guard.get_exchange(&order.account_id).await;
                
                // 获取订单信息
                let latest_order = exchange.update_order(order.clone()).await.expect("更新订单失败");
                tracing::info!("最新订单: {:?}", latest_order);
                
                // 如果订单状态为已成交，则通知持仓引擎，订单已成交
                if latest_order.order_status == OrderStatus::Filled {
                    //1. 先通知持仓引擎和交易明细引擎，订单已成交
                    let order_event = OrderEvent::OrderFilled(latest_order.clone());
                    event_publisher.publish(order_event.into()).unwrap();
                    

                    // 2. 未成交订单列表中删除
                    let mut unfilled_orders = unfilled_orders.write().await;
                    // 删除订单，使用latest_order的ID而不是原始order的ID
                    tracing::info!("订单已成交, 从未成交订单列表中删除: {:?}", latest_order);
                    unfilled_orders.entry(strategy_id.clone()).and_modify(|orders| {
                        orders.retain(|o| o.order_id != latest_order.order_id); // 只删除order_id相同的订单
                    });

                    // 3.更新数据库订单信息
                    OrderMutation::update_order(&database, latest_order.clone()).await.unwrap();

                    
                }
                

            }
        }
    }

    pub async fn monitor_unfilled_orders(&mut self) {

        let unfilled_orders = self.unfilled_orders.clone();
        let exchange_engine = self.exchange_engine.clone();
        let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控未成交订单".to_string(),
            move || {
                let unfilled_orders = unfilled_orders.clone();
                let exchange_engine = exchange_engine.clone();
                let event_publisher = event_publisher.clone();
                let database = database.clone();
                async move {
                    Self::process_unfilled_orders(
                        unfilled_orders,
                        exchange_engine,
                        event_publisher,
                        database
                    ).await
                }
            },
            10
        ).await;
        
    }

    

}