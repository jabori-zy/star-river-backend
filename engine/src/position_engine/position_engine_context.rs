use tokio::sync::broadcast;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use std::sync::Arc;
use event_center::EventPublisher;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use event_center::order_event::OrderEvent;
use types::order::Order;
use event_center::command_event::position_engine_command::GetPositionParam;
use types::position::Position;
use database::mutation::position_mutation::PositionMutation;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::collections::HashMap;
use tokio::sync::RwLock;


#[derive(Debug)]
pub struct PositionEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub positions: Arc<RwLock<HashMap<i64, Vec<Position>>>>, // 持仓, 策略id作为key
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>
}

impl Clone for PositionEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            exchange_engine: self.exchange_engine.clone(),
            positions: self.positions.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),

        }
    }
}


#[async_trait]
impl EngineContext for PositionEngineContext {
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
                        self.handle_order_filled(order).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}

        }

    }
}


impl PositionEngineContext {
    async fn handle_order_filled(&mut self, order: Order) -> Result<(), String> {
        tracing::info!("订单已成交, 获取持仓: {:?}", order);

        // 1. 先检查交易所注册状态
        let is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&order.exchange).await
        };

        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", &order.exchange));
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

        let exchange = exchange_engine_context_guard.get_exchange_ref(&order.exchange).await.unwrap();

        let get_position_params = GetPositionParam {
            strategy_id: order.strategy_id.clone(),
            node_id: order.node_id.clone(),
            exchange: order.exchange.clone(),
            position_id: order.exchange_order_id.clone(),
        };

        let exchange_position= exchange.get_position(get_position_params).await.unwrap();
        tracing::info!("获取持仓: {:?}", exchange_position);
        // 入库
        if let Ok(position) = PositionMutation::insert_position(&self.database, order.strategy_id.clone(), order.node_id.clone(), exchange_position.clone()).await {
            tracing::info!("订单入库成功: {:?}", position);
            // 将订单添加到未成交订单列表
            let mut positions = self.positions.write().await;
            positions.entry(order.strategy_id).or_insert(vec![]).push(position);
            
        } else {
            tracing::error!("订单入库失败: {:?}", exchange_position);
            return Err("订单入库失败".to_string());
        }


        Ok(())
    }


    pub async fn monitor_positions(&mut self) {
        let positions = self.positions.clone();
        let exchange_engine = self.exchange_engine.clone();
        let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控持仓".to_string(),
            move || {
                let positions = positions.clone();
                let exchange_engine = exchange_engine.clone();
                let event_publisher = event_publisher.clone();
                let database = database.clone();
                async move {
                    Self::process_positions(
                        positions,
                        exchange_engine,
                        event_publisher,
                        database
                    ).await
                }
            },
            10
        ).await;
    }

    async fn process_positions(
        positions: Arc<RwLock<HashMap<i64, Vec<Position>>>>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        event_publisher: EventPublisher,
        database: DatabaseConnection,
    ) {
        let positions_clone = {
            let positions = positions.read().await;
            positions.clone()
        };

        // 如果hashmap为空，则直接返回
        if positions_clone.is_empty() {
            return;
        }

        // 遍历持仓
        for (strategy_id, positions) in positions_clone.iter() {
            // 先判断列表的长度
            // 如果策略的订单列表为空, 则跳过
            if positions.len() == 0 {
                continue;
            }
            // 列表不为空, 则遍历列表
            for position in positions {
                // 获取交易所的上下文
                let exchange_engine_guard = exchange_engine.lock().await;
                // 获取交易所对象
                let exchange = exchange_engine_guard.get_exchange(&position.exchange).await;
                
                // 获取持仓信息
                let latest_position = exchange.update_position(position).await.expect("更新订单失败");
                tracing::info!("未平仓利润: {:?}", latest_position.unrealized_profit);
                
                
                
            }
        }

        
    }

}

