use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use super::position_management_node_types::*;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use types::strategy::node_event::{BacktestNodeEvent, OrderEvent};
use types::order::Order;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use exchange_client::ExchangeClient;
use crate::Engine;
use database::mutation::position_mutation::PositionMutation;
use types::position::GetPositionParam;
use types::strategy::node_event::PositionEvent;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use types::strategy::node_event::SignalEvent;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use utils::get_utc8_timestamp_millis;
use virtual_trading::VirtualTradingSystem;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::virtual_trading_system::event::{VirtualTradingSystemEvent, VirtualTradingSystemEventReceiver};
use types::strategy::node_event::backtest_node_event::position_management_node_event::{PositionCreatedEvent, PositionUpdatedEvent, PositionClosedEvent, PositionManagementNodeEvent};
use types::custom_type::PlayIndex;
use types::position::virtual_position::VirtualPosition;
use types::strategy::node_event::ExecuteOverEvent;

#[derive(Debug)]
pub struct PositionNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: PositionNodeBacktestConfig,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 持仓, 策略id作为key
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
}

impl Clone for PositionNodeContext {
    fn clone(&self) -> Self {
        Self {
            base_context: self.base_context.clone(),
            backtest_config: self.backtest_config.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            virtual_trading_system_event_receiver: self.virtual_trading_system_event_receiver.resubscribe(),
        }
    }
}






#[async_trait]
impl BacktestNodeContextTrait for PositionNodeContext {
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
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
        self.base_context.output_handles.get(&format!("position_node_update_output")).unwrap().clone()
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), String> {
        // tracing::info!("{}: 收到节点事件: {:?}", self.get_node_name(), node_event);

        match node_event {
            BacktestNodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::BacktestConditionNotMatch(_) => {
                        tracing::debug!("{}: 条件不匹配，不获取仓位信息。节点是否是叶子节点: {}", self.get_node_name(), self.is_leaf_node());
                        if self.is_leaf_node() {
                            let execute_over_event = ExecuteOverEvent {
                                from_node_id: self.get_node_id().clone(),
                                from_node_name: self.get_node_name().clone(),
                                from_node_handle_id: self.get_node_id().clone(),
                                play_index: self.get_play_index(),
                                timestamp: get_utc8_timestamp_millis(),
                            };
                            let strategy_output_handle = self.get_strategy_output_handle();
                            strategy_output_handle.send(BacktestNodeEvent::Signal(SignalEvent::ExecuteOver(execute_over_event))).unwrap();
                        }
                    }
                    _ => {}
                    
                    
                }
            }

            BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) => {
                tracing::debug!("{}: 收到订单事件: {:?}", self.get_node_name(), futures_order_node_event);
                if self.is_leaf_node() {
                    let execute_over_event = ExecuteOverEvent {
                        from_node_id: self.get_node_id().clone(),
                        from_node_name: self.get_node_name().clone(),
                        from_node_handle_id: self.get_node_id().clone(),
                        play_index: self.get_play_index(),
                        timestamp: get_utc8_timestamp_millis(),
                    };
                    let strategy_output_handle = self.get_strategy_output_handle();
                    strategy_output_handle.send(BacktestNodeEvent::Signal(SignalEvent::ExecuteOver(execute_over_event))).unwrap();
                }
            }

            _ => {}
        }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        // match strategy_inner_event {
        //     StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
        //         // 更新k线缓存索引
        //         self.set_play_index(play_index_update_event.play_index).await;
        //         let strategy_output_handle_id = format!("{}_strategy_output", self.get_node_id());
        //         let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
        //             from_node_id: self.get_node_id().clone(),
        //             from_node_name: self.get_node_name().clone(),
        //             from_node_handle_id: strategy_output_handle_id.clone(),
        //             play_index: self.get_play_index().await,
        //             message_timestamp: get_utc8_timestamp_millis(),
        //         }));
        //         self.get_strategy_output_handle().send(signal).unwrap();
        //     }
        //     _ => {}  
        // }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        Ok(())
    }



}


impl PositionNodeContext {
    pub async fn handle_virtual_trading_system_event(&mut self, virtual_trading_system_event: VirtualTradingSystemEvent) -> Result<(), String> {

        let from_node_id = self.get_node_id().clone();
        let from_node_name = self.get_node_name().clone();
        let from_handle_id = self.get_node_id().clone();

        let position_event: Option<PositionManagementNodeEvent> = match virtual_trading_system_event {
            VirtualTradingSystemEvent::PositionCreated(position) => {
                let position_created_event = PositionManagementNodeEvent::PositionCreated(PositionCreatedEvent {
                    from_node_id: from_node_id.clone(),
                    from_node_name: from_node_name.clone(),
                    from_handle_id: from_handle_id.clone(),
                    virtual_position: position,
                    timestamp: get_utc8_timestamp_millis(),
                });
                Some(position_created_event)
            }
            VirtualTradingSystemEvent::PositionUpdated(position) => {
                let position_updated_event = PositionManagementNodeEvent::PositionUpdated(PositionUpdatedEvent {
                    from_node_id: from_node_id.clone(),
                    from_node_name: from_node_name.clone(),
                    from_handle_id: from_handle_id.clone(),
                    virtual_position: position,
                    timestamp: get_utc8_timestamp_millis(),
                });
                Some(position_updated_event)
            }
            VirtualTradingSystemEvent::PositionClosed(position) => {
                let position_closed_event = PositionManagementNodeEvent::PositionClosed(PositionClosedEvent {
                    from_node_id: from_node_id.clone(),
                    from_node_name: from_node_name.clone(),
                    from_handle_id: from_handle_id.clone(),
                    virtual_position: position,
                    timestamp: get_utc8_timestamp_millis(),
                });
                Some(position_closed_event)
            }
            _ => None,
        };

        if let Some(position_event) = position_event {
            let strategy_output_handle = self.get_strategy_output_handle();
            strategy_output_handle.send(BacktestNodeEvent::PositionManagementNode(position_event)).unwrap();
        }

        Ok(())
    }
    // async fn get_exchange(&self, account_id: &i32) -> Result<Box<dyn ExchangeClient>, String> {
    //     // 1. 先检查交易所注册状态
    //     let is_registered = {
    //         let exchange_engine_guard = self.exchange_engine.lock().await;
    //         exchange_engine_guard.is_registered(account_id).await
    //     };

    //     if !is_registered {
    //         return Err("交易所未注册".to_string());
    //     }

    //     // 2. 获取上下文（新的锁范围）
    //     let exchange_engine_context = {
    //         let exchange_engine_guard = self.exchange_engine.lock().await;
    //         exchange_engine_guard.get_context()
    //     };

    //     // 3. 获取读锁
    //     let context_read = exchange_engine_context.read().await;
    //     let exchange_engine_context_guard = context_read
    //         .as_any()
    //         .downcast_ref::<ExchangeEngineContext>()
    //         .unwrap();

    //     exchange_engine_context_guard.get_exchange(account_id).await
    // }

    // async fn get_position(&mut self, order: Order) -> Result<(), String> {
    //     // 订单已完成，获取持仓
    //     tracing::info!("订单已完成，获取持仓: {:?}", order);
        
    //     // 使用 as_ref().ok_or() 代替 unwrap
    //     let account_id = self.backtest_config.selected_account.account_id;

    //     // 使用 ? 操作符代替 unwrap
    //     let exchange = self.get_exchange(&order.account_id).await?;

    //     let get_position_params = GetPositionParam {
    //         strategy_id: order.strategy_id.clone(),
    //         node_id: order.node_id.clone(),
    //         exchange: order.exchange.clone(),
    //         position_id: order.exchange_order_id.clone(),
    //     };

    //     // 使用 ? 操作符代替 unwrap
    //     let exchange_position = exchange.get_position(get_position_params).await;
    //     match exchange_position {
    //         Ok(position) => {
    //             tracing::info!("获取持仓: {:?}", position);
    //             // // 入库
    //             let position = PositionMutation::insert_position(
    //                 &self.database, 
    //                 order.strategy_id.clone() as i64, 
    //                 order.node_id.clone(), 
    //                 account_id, 
    //                 position.clone()
    //             ).await;
                
    //             // // 使用 map 和 ? 操作符而不是 unwrap
    //             let position = position.map_err(|e| {
    //                 tracing::error!("仓位信息入库失败: {:?}", e);
    //                 e.to_string()
    //             })?;

    //             // 发送仓位更新消息
    //             let output_handle = self.get_all_output_handle().get("position_node_update_output").unwrap();

    //             let position_event = PositionEvent::PositionUpdated(position);
    //             tracing::debug!("{}: 发送仓位更新消息: {:?}", self.get_node_name(), position_event);
    //             output_handle.send(NodeEvent::Position(position_event)).unwrap();
    //         }
    //         Err(_) => {
    //             tracing::warn!("仓位已关闭: {:?}", order.exchange_order_id);
    //         }
    //     }
        
        

    //     Ok(())
    // }
}

