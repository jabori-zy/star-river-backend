use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use chrono::Utc;
use event_center::Event;
use uuid::Uuid;
use crate::strategy_engine::node::node_context::{LiveBaseNodeContext,LiveNodeContextTrait};
use types::strategy::node_message::NodeMessage;
use types::strategy::node_message::SignalType;
use event_center::response::Response;
use event_center::command::Command;
use super::order_node_types::*;
use types::strategy::TradeMode;
use tokio::sync::Mutex;
use crate::Engine;
use crate::exchange_engine::ExchangeEngine;
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use types::order::Order;
use types::order::OrderStatus;
use database::mutation::order_mutation::OrderMutation;
use tokio::sync::RwLock;
use types::strategy::node_message::OrderMessage;
use database::mutation::transaction_mutation::TransactionMutation;
use exchange_client::ExchangeClient;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use types::order::{CreateOrderParams,GetTransactionDetailParams};

#[derive(Debug, Clone)]
pub struct OrderNodeContext {
    pub base_context: LiveBaseNodeContext,
    pub live_config: OrderNodeLiveConfig,
    pub request_id: Vec<Uuid>,
    pub is_processing_order: Arc<RwLock<bool>>, // 是否正在处理订单
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>, // 交易所引擎
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub unfilled_order: Arc<RwLock<Option<Order>>>, // 未成交的订单
}

impl OrderNodeContext {
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
        let transaction = TransactionMutation::insert_transaction(
            &self.database,
            order.strategy_id.clone() as i64,
            order.node_id,
            transaction_detail,
        ).await.unwrap();
        tracing::info!("交易明细入库成功: {:?}", transaction);

        Ok(())
    }

    
    async fn set_is_processing_order(&mut self, is_processing_order: bool) {
        *self.is_processing_order.write().await = is_processing_order;
    }

    async fn set_unfilled_order(&mut self, unfilled_order: Option<Order>) {
        *self.unfilled_order.write().await = unfilled_order;
    }

    async fn create_order(&mut self) {
        // 如果当前是正在处理订单的状态，则不创建订单
        if *self.is_processing_order.read().await || self.unfilled_order.read().await.is_some() {
            // tracing::warn!("{}: 当前正在处理订单, 跳过", self.get_node_name());
            return;
        }

        // 将is_processing_order设置为true
        self.set_is_processing_order(true).await;
        tracing::info!("{}: 开始创建订单", self.get_node_id());
        
        let exchange = self.get_exchange(&self.live_config.selected_live_account.account_id).await.unwrap();
        let create_order_params = CreateOrderParams {
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            account_id: self.live_config.selected_live_account.account_id,
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.order_config.symbol.clone(),
            order_type: self.live_config.order_config.order_type.clone(),
            order_side: self.live_config.order_config.order_side.clone(),
            quantity: self.live_config.order_config.quantity,
            price: self.live_config.order_config.price,
            tp: self.live_config.order_config.tp,
            sl: self.live_config.order_config.sl,
            comment: "111".to_string(),

        };
        let original_order = exchange.create_order(create_order_params.clone()).await.unwrap();
        if let Ok(order) = OrderMutation::insert_order(&self.database, self.get_strategy_id().clone() as i64, self.get_node_id().clone(), self.live_config.selected_live_account.account_id, original_order.clone()).await {
            tracing::info!("订单入库成功: {:?}", order);
            // 如果订单状态为已成交，则通知持仓引擎，订单已成交
            if original_order.get_order_status() == OrderStatus::Filled {
                // 发送订单已成交信号
                let output_handle = self.get_all_output_handle().get("order_node_output").unwrap();
                let order_message = OrderMessage::OrderFilled(order.clone());
                // 发送消息
                output_handle.send(NodeMessage::Order(order_message.clone())).unwrap();
                // 获取交易明细
                self.get_transaction_detail(order).await.unwrap();

            } 
            // 如果订单状态为其他的状态，则将订单添加为正在处理的订单
            else {
                // 将订单添加为正在处理的订单
                self.set_unfilled_order(Some(order)).await;
            }
        } else {
            tracing::error!("订单入库失败: {:?}", original_order);
        }
    }

    async fn send_test_signal(&mut self) {
        let output_handle = self.get_all_output_handle().get("order_node_output").unwrap();
        let order_message = OrderMessage::OrderFilled(Order {
            order_id: 1,
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            exchange_order_id: 475265246,
            account_id: self.live_config.selected_live_account.account_id,
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.order_config.symbol.clone(),
            order_side: self.live_config.order_config.order_side.clone(),
            order_type: self.live_config.order_config.order_type.clone(),
            order_status: OrderStatus::Filled,
            quantity: self.live_config.order_config.quantity,
            open_price: self.live_config.order_config.price,
            tp: self.live_config.order_config.tp,
            sl: self.live_config.order_config.sl,
            extra_info: Some(serde_json::to_value("111".to_string()).unwrap()),
            created_time: Utc::now(),
            updated_time: Utc::now(),
        });
        output_handle.send(NodeMessage::Order(order_message.clone())).unwrap();
    }

    pub async fn monitor_unfilled_order(&mut self) {
        let unfilled_order = self.unfilled_order.clone();
        let is_processing_order = self.is_processing_order.clone();
        let exchange_engine = self.exchange_engine.clone();
        let database = self.database.clone();
        let node_name = self.get_node_name().clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            format!("{}监控未成交订单", self.get_node_name()),
            move || {
                let unfilled_order = unfilled_order.clone();
                let is_processing_order = is_processing_order.clone();
                let exchange_engine = exchange_engine.clone();
                let database = database.clone();
                let node_name = node_name.clone();
                async move {
                    Self::process_unfilled_order(
                        node_name,
                        unfilled_order,
                        is_processing_order,
                        exchange_engine,
                        database
                    ).await
                }
            },
            10
        ).await;
    }

    async fn process_unfilled_order(
        node_name: String,
        unfilled_order: Arc<RwLock<Option<Order>>>,
        is_processing_order: Arc<RwLock<bool>>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
    ) {
        let unfilled_order_clone = {
            let unfilled_order_guard = unfilled_order.read().await;
            unfilled_order_guard.clone()
        };
        // 如果当前没有正在处理的订单，则直接返回
        if unfilled_order_clone.is_none() {
            // tracing::info!("{}: 没有未成交订单", node_name);
            return;
        }
        // 如果当前有正在处理的订单，则获取订单的最新状态
        if let Some(order) = unfilled_order_clone {
            let exchange_engine_guard = exchange_engine.lock().await;
            let exchange = exchange_engine_guard.get_exchange(&order.account_id).await;
            match exchange {
                Ok(exchange) => {
                    let latest_order = exchange.update_order(order.clone()).await.expect("更新订单失败");
                    tracing::info!("最新订单: {:?}", latest_order);
                    if latest_order.order_status == OrderStatus::Filled {
                        // 更新数据库
                        OrderMutation::update_order(&database, latest_order.clone()).await.unwrap();
                        // 删除未成交的订单
                        let mut unfilled_order_guard = unfilled_order.write().await;
                        // 使用take方法，将订单从Option中移除
                        unfilled_order_guard.take();
                        // 设置is_processing_order为false
                        *is_processing_order.write().await = false;
                        

                    }
                }
                Err(e) => {
                    tracing::error!("获取交易所客户端失败: {:?}", e);
                }
            }
        }


    }


}

#[async_trait]
impl LiveNodeContextTrait for OrderNodeContext {
    fn clone_box(&self) -> Box<dyn LiveNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &LiveBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut LiveBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("order_node_output")).unwrap().clone()
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::Signal(signal_message) => {
                tracing::debug!("{}: 收到信号: {:?}", self.get_node_name(), signal_message);

                match signal_message.signal_type {
                    // 如果信号为True，则执行下单
                    SignalType::ConditionMatch => {
                        // self.create_order().await;
                        self.create_order().await;
                        // self.send_test_signal().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}


