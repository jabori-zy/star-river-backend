use crate::strategy_engine::node::node_context::{BaseNodeContext,NodeContextTrait};
use super::position_node_types::*;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use types::strategy::node_message::{NodeMessage, OrderMessage};
use types::order::Order;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use exchange_client::ExchangeClient;
use crate::Engine;
use database::mutation::position_mutation::PositionMutation;
use types::position::GetPositionParam;
use types::strategy::node_message::PositionMessage;
use crate::strategy_engine::node::node_types::NodeOutputHandle;

#[derive(Debug, Clone)]
pub struct PositionNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: PositionNodeLiveConfig,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 持仓, 策略id作为key
}




#[async_trait]
impl NodeContextTrait for PositionNodeContext {
    fn clone_box(&self) -> Box<dyn NodeContextTrait> {
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
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("position_node_update_output")).unwrap().clone()
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::Order(order_message) => {
                match order_message {
                    OrderMessage::OrderFilled(order) => {
                        tracing::debug!("{}: 收到订单已完成信息: {:?}", self.get_node_name(), order);
                        if let Err(e) = self.get_position(order).await {
                            tracing::error!("{}: 获取仓位信息失败: {:?}", self.get_node_name(), e);
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


impl PositionNodeContext {
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

    async fn get_position(&mut self, order: Order) -> Result<(), String> {
        // 订单已完成，获取持仓
        tracing::info!("订单已完成，获取持仓: {:?}", order);
        
        // 使用 as_ref().ok_or() 代替 unwrap
        let account_id = self.live_config.selected_live_account.account_id;

        // 使用 ? 操作符代替 unwrap
        let exchange = self.get_exchange(&order.account_id).await?;

        let get_position_params = GetPositionParam {
            strategy_id: order.strategy_id.clone(),
            node_id: order.node_id.clone(),
            exchange: order.exchange.clone(),
            position_id: order.exchange_order_id.clone(),
        };

        // 使用 ? 操作符代替 unwrap
        let exchange_position = exchange.get_position(get_position_params).await;
        match exchange_position {
            Ok(position) => {
                tracing::info!("获取持仓: {:?}", position);
                // // 入库
                let position = PositionMutation::insert_position(
                    &self.database, 
                    order.strategy_id.clone() as i64, 
                    order.node_id.clone(), 
                    account_id, 
                    position.clone()
                ).await;
                
                // // 使用 map 和 ? 操作符而不是 unwrap
                let position = position.map_err(|e| {
                    tracing::error!("仓位信息入库失败: {:?}", e);
                    e.to_string()
                })?;

                // 发送仓位更新消息
                let output_handle = self.get_all_output_handle().get("position_node_update_output").unwrap();

                let position_message = PositionMessage::PositionUpdated(position);
                tracing::debug!("{}: 发送仓位更新消息: {:?}", self.get_node_name(), position_message);
                output_handle.send(NodeMessage::Position(position_message)).unwrap();
            }
            Err(_) => {
                tracing::warn!("仓位已关闭: {:?}", order.exchange_order_id);
            }
        }
        
        

        Ok(())
    }
}

