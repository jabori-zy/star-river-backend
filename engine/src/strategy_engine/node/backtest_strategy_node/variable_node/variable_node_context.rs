use crate::strategy_engine::node::{node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait}};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use types::node::variable_node::*;
use types::strategy::sys_varibale::SysVariable;
use database::query::strategy_sys_variable_query::StrategySysVariableQuery;
use types::strategy::node_event::{SignalEvent, VariableMessage, PlayIndexUpdateEvent};
use utils::get_utc8_timestamp_millis;
use types::strategy::node_event::NodeEvent;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::collections::HashMap;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::node::variable_node::GetVariableType;
use virtual_trading::VirtualTradingSystem;


#[derive(Debug, Clone)]
pub struct VariableNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: VariableNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub database: DatabaseConnection,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
}


#[async_trait]
impl BacktestNodeContextTrait for VariableNodeContext {
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

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handles.get(&format!("get_variable_node_output")).unwrap().clone()
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 处理事件: {:?}", self.get_node_id(), event);
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    async fn handle_node_event(&mut self, node_event: NodeEvent) -> Result<(), String> {
        // match node_event {
        //     NodeEvent::Signal(SignalEvent::BacktestConditionMatch(condition_match_event)) => {
        //         // 判断当前节点的模式
        //         // 如果是条件触发模式，则获取变量
        //         if self.backtest_config.variable_configs.iter().any(|v| v.get_variable_type == GetVariableType::Condition) {
        //             tracing::info!("{}: 条件触发模式，获取变量", self.get_node_name());
        //             self.get_variable().await;

        //         }

        //     }

        //     _ => {}

        // }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新k线缓存索引
                self.set_play_index(play_index_update_event.played_index).await;
                let strategy_output_handle = self.get_strategy_output_handle();
                // tracing::debug!("{}: 更新k线缓存索引: {}", self.get_node_id(), play_index_update_event.played_index);
                let signal = NodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
                    node_play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp_millis(),
                }));
                strategy_output_handle.send(signal).unwrap();
            }
            _ => {}
        }
        
        Ok(())
    }

}

impl VariableNodeContext {
    pub async fn register_task(&mut self) {
        // let database = self.database.clone();
        // let backtest_config = self.backtest_config.clone();
        // let timer_config = backtest_config.timer_config.unwrap();
        // let variables = backtest_config.variables.clone();
        // let node_name = self.get_node_name().clone();
        // let strategy_id = self.get_strategy_id().clone();
        // let node_id = self.get_node_id().clone();
        // let all_output_handle = self.get_all_output_handle().clone();

        // let mut heartbeat = self.heartbeat.lock().await;
        // heartbeat.register_async_task(format!("{}: 注册处理变量任务", node_name),
        // move || {
        //     let strategy_id = strategy_id.clone();
        //     let node_id = node_id.clone();
        //     let node_name = node_name.clone();
        //     let variables = variables.clone();
        //     let database = database.clone();
        //     let all_output_handle = all_output_handle.clone();
        //     async move {
        //         Self::process_variable(
        //             strategy_id, 
        //             node_id,
        //             node_name, 
        //             variables, 
        //             database, 
        //             all_output_handle).await
        //         }
        //     },
        //     timer_config.get_millisecond()/100
        // ).await;
    }

    
    pub async fn get_variable(&mut self) {
        // let variables = self.backtest_config.variables.clone();

        // for var in variables {
        //     let variable_type = var.variable.clone();
        //     match variable_type {
        //         SysVariable::PositionNumber => {
        //             let position_number = self.get_position_number().await;
        //             let variable_message = VariableMessage {
        //                 from_node_id: self.get_node_id().clone(),
        //                 from_node_name: self.get_node_name().clone(),
        //                 from_node_handle_id: var.config_id.clone(),
        //                 variable: var.variable.to_string(),
        //                 variable_value: position_number as f64,
        //                 message_timestamp: get_utc8_timestamp_millis(),
        //             };
        //             let output_handle = self.get_all_output_handle().get(&var.config_id).unwrap();
        //             tracing::debug!("{}: 发送仓位数量更新事件: {:?}", self.get_node_id(), variable_message);
        //             output_handle.send(NodeEvent::Variable(variable_message)).unwrap();
        //         }
        //         _ => {}
        //     }
        // }
    }
    
    async fn process_variable(
        strategy_id: i32,
        node_id: String,
        node_name: String, 
        variables: Vec<VariableConfig>, 
        database: DatabaseConnection,
        output_handle: HashMap<String, NodeOutputHandle>,

    ) {
        
        for var in variables {
            let variable_type = var.variable.clone();
            match variable_type {
                SysVariable::PositionNumber => {
                }
                _ => {}
            }
        }
        
        
    }

    async fn get_position_number(&self) -> u32 {
        // let virtual_trading_system = self.virtual_trading_system.lock().await;
        // let exchange = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_data_source.exchange.clone();
        // let symbol = self.backtest_config.exchange_mode_config.as_ref().unwrap().symbol.clone();
        // let position_number = virtual_trading_system.get_symbol_position_number(&symbol, &exchange);
        // position_number
        0
    }
}

