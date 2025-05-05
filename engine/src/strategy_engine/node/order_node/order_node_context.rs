use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use chrono::Utc;
use event_center::Event;
use uuid::Uuid;
use super::super::node_context::{BaseNodeContext,NodeContext};
use types::strategy::message::NodeMessage;
use types::strategy::message::{SignalMessage,SignalType,Signal};
use event_center::response_event::ResponseEvent;
use event_center::command_event::CommandEvent;
use event_center::command_event::order_engine_command::OrderEngineCommand;
use event_center::command_event::order_engine_command::CreateOrderParams;
use event_center::command_event::BaseCommandParams;
use super::order_node_types::*;
use types::strategy::TradeMode;
use event_center::response_event::order_engine_response::{OrderEngineResponse, CreateOrderResponse};
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
use types::strategy::message::OrderMessage;
use database::mutation::transaction_mutation::TransactionMutation;
use event_center::command_event::order_engine_command::GetTransactionDetailParams;
use exchange_client::ExchangeClient;

#[derive(Debug, Clone)]
pub struct OrderNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: Option<OrderNodeLiveConfig>,
    pub simulate_config: Option<OrderNodeSimulateConfig>,
    pub backtest_config: Option<OrderNodeBacktestConfig>,
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
            order.strategy_id,
            order.node_id,
            transaction_detail,
        ).await.unwrap();
        tracing::info!("交易明细入库成功: {:?}", transaction);

        Ok(())
    }
    async fn handle_response_event(&mut self, response_event: ResponseEvent){
        // 如果request_id列表为空，说明没有请求，不处理，直接返回
        if self.request_id.is_empty() {
            return;
        }
        // 如果request_id列表不为空，则处理响应
        match response_event {
            // 订单引擎创建，创建订单成功的响应
            ResponseEvent::OrderEngine(OrderEngineResponse::CreateOrderResponse(create_order_response)) => {
                // 如果response_id在request_id列表中，则先处理，再删除request_id
                if self.request_id.contains(&create_order_response.response_id) {
                    // tracing::info!("{}: 订单创建成功: {:?}", self.get_node_id(), create_order_response);
                    // self.request_id.remove(self.request_id.iter().position(|id| *id == create_order_response.response_id).unwrap());
                    self.handle_create_order_response(create_order_response).await;
                }
            }
            _ => {}
        }
    }

    async fn handle_create_order_response(&mut self, create_order_response: CreateOrderResponse) {
        if create_order_response.code == 0 {
            // 订单创建成功
            tracing::info!("{}: 订单创建成功: {:?}", self.get_node_id(), create_order_response);
            *self.is_processing_order.write().await = false; // 设置为false，表示订单创建成功，可以创建下一个订单
            // 如果订单创建成功，则将request_id从列表中删除
            self.request_id.remove(self.request_id.iter().position(|id| *id == create_order_response.response_id).unwrap());
        } else {
            // 订单未成交
            tracing::warn!("{}: 订单未成交: {:?}", self.get_node_id(), create_order_response);
        }
    }

    async fn create_order(&mut self) {
        if *self.is_processing_order.read().await {
            tracing::warn!("{}: 正在处理订单, 跳过", self.get_node_id());
            return;
        }
        *self.is_processing_order.write().await = true;
        match self.base_context.trade_mode {
            TradeMode::Live => {
                let request_id = Uuid::new_v4();
                self.request_id.push(request_id); // 将request_id添加到request_id列表中
                let create_order_params = CreateOrderParams {
                    base_params: BaseCommandParams {
                        strategy_id: self.get_strategy_id().clone(),
                        node_id: self.get_node_id().clone(),
                        sender: self.get_node_id().clone(),
                        timestamp: get_utc8_timestamp_millis(),
                        request_id: request_id,
                    },
                    account_id: self.live_config.as_ref().unwrap().selected_live_account.account_id,
                    exchange: self.live_config.as_ref().unwrap().selected_live_account.exchange.clone(),
                    symbol: self.live_config.as_ref().unwrap().order_config.symbol.clone(),
                    order_type: self.live_config.as_ref().unwrap().order_config.order_type.clone(),
                    order_side: self.live_config.as_ref().unwrap().order_config.order_side.clone(),
                    quantity: self.live_config.as_ref().unwrap().order_config.quantity,
                    price: self.live_config.as_ref().unwrap().order_config.price,
                    tp: self.live_config.as_ref().unwrap().order_config.tp,
                    sl: self.live_config.as_ref().unwrap().order_config.sl,
                    comment: "111".to_string(),
        
                };

                tracing::info!("{}: 发送创建订单命令: {:?}", self.get_node_id(), create_order_params);
                let command_event = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(create_order_params));
                self.get_event_publisher().publish(command_event.into()).expect("发送创建订单命令失败");
                


            }
            _ => {
                tracing::error!("{}: 暂不支持的TradeMode: {:?}", self.get_node_id(), self.base_context.trade_mode);
            }
        }
        
    }

    
    async fn set_is_processing_order(&mut self, is_processing_order: bool) {
        *self.is_processing_order.write().await = is_processing_order;
    }

    async fn set_unfilled_order(&mut self, unfilled_order: Option<Order>) {
        *self.unfilled_order.write().await = unfilled_order;
    }

    async fn create_order2(&mut self) {
        // 如果当前是正在处理订单的状态，则不创建订单
        if *self.is_processing_order.read().await || self.unfilled_order.read().await.is_some() {
            // tracing::warn!("{}: 当前正在处理订单, 跳过", self.get_node_name());
            return;
        }

        // 将is_processing_order设置为true
        self.set_is_processing_order(true).await;
        

        match self.base_context.trade_mode {
            TradeMode::Live => {
                tracing::info!("{}: 开始创建订单", self.get_node_id());
                
                let exchange = self.get_exchange(&self.live_config.as_ref().unwrap().selected_live_account.account_id).await.unwrap();
                let create_order_params = CreateOrderParams {
                    base_params: BaseCommandParams {
                        strategy_id: self.get_strategy_id().clone(),
                        node_id: self.get_node_id().clone(),
                        sender: self.get_node_id().clone(),
                        timestamp: get_utc8_timestamp_millis(),
                        request_id: Uuid::new_v4(),
                    },
                    account_id: self.live_config.as_ref().unwrap().selected_live_account.account_id,
                    exchange: self.live_config.as_ref().unwrap().selected_live_account.exchange.clone(),
                    symbol: self.live_config.as_ref().unwrap().order_config.symbol.clone(),
                    order_type: self.live_config.as_ref().unwrap().order_config.order_type.clone(),
                    order_side: self.live_config.as_ref().unwrap().order_config.order_side.clone(),
                    quantity: self.live_config.as_ref().unwrap().order_config.quantity,
                    price: self.live_config.as_ref().unwrap().order_config.price,
                    tp: self.live_config.as_ref().unwrap().order_config.tp,
                    sl: self.live_config.as_ref().unwrap().order_config.sl,
                    comment: "111".to_string(),
        
                };
                let original_order = exchange.create_order(create_order_params.clone()).await.unwrap();
                if let Ok(order) = OrderMutation::insert_order(&self.database, self.get_strategy_id().clone(), self.get_node_id().clone(), self.live_config.as_ref().unwrap().selected_live_account.account_id, original_order.clone()).await {
                    tracing::info!("订单入库成功: {:?}", order);
                    // 如果订单状态为已成交，则通知持仓引擎，订单已成交
                    if original_order.get_order_status() == OrderStatus::Filled {
                        // 发送订单已成交信号
                        let output_handle = self.get_output_handle().get("order_node_output").unwrap();
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
            _ => {
                tracing::error!("{}: 暂不支持的TradeMode: {:?}", self.get_node_id(), self.base_context.trade_mode);
            }
        }
    }

    async fn send_test_signal(&mut self) {
        let output_handle = self.get_output_handle().get("order_node_output").unwrap();
        let order_message = OrderMessage::OrderFilled(Order {
            order_id: 1,
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            exchange_order_id: 475265246,
            account_id: self.live_config.as_ref().unwrap().selected_live_account.account_id,
            exchange: self.live_config.as_ref().unwrap().selected_live_account.exchange.clone(),
            symbol: self.live_config.as_ref().unwrap().order_config.symbol.clone(),
            order_side: self.live_config.as_ref().unwrap().order_config.order_side.clone(),
            order_type: self.live_config.as_ref().unwrap().order_config.order_type.clone(),
            order_status: OrderStatus::Filled,
            quantity: self.live_config.as_ref().unwrap().order_config.quantity,
            open_price: self.live_config.as_ref().unwrap().order_config.price,
            tp: self.live_config.as_ref().unwrap().order_config.tp,
            sl: self.live_config.as_ref().unwrap().order_config.sl,
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
impl NodeContext for OrderNodeContext {
    fn clone_box(&self) -> Box<dyn NodeContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Response(response_event) => {
                self.handle_response_event(response_event).await;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::Signal(signal_message) => {
                tracing::debug!("{}: 收到信号: {:?}", self.get_node_name(), signal_message);

                match signal_message.signal {
                    // 如果信号为True，则执行下单
                    Signal::True => {
                        // self.create_order().await;
                        if signal_message.signal_type == SignalType::ConditionMatch {
                            self.create_order2().await;
                            // self.send_test_signal().await;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}


