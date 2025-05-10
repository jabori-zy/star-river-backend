
use crate::strategy_engine::node::node_context::{BaseNodeContext,NodeContext};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use super::get_variable_node_types::*;
use types::strategy::sys_varibale::SysVariable;
use database::query::strategy_sys_variable_query::StrategySysVariableQuery;
use types::strategy::node_message::VariableMessage;
use utils::get_utc8_timestamp_millis;
use types::strategy::node_message::NodeMessage;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::collections::HashMap;
use types::strategy::node_message::SignalType;

#[derive(Debug, Clone)]
pub struct GetVariableNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: GetVariableNodeLiveConfig,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub database: DatabaseConnection,
}


#[async_trait]
impl NodeContext for GetVariableNodeContext {
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

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("get_variable_node_output")).unwrap().clone()
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
                tracing::info!("{}: 收到信号: {:?}", self.get_node_name(), signal_message.signal_type);
                match signal_message.signal_type {
                    // 如果信号为True，则执行下单
                    SignalType::ConditionMatch => {
                        self.get_variable().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}

impl GetVariableNodeContext {
    pub async fn register_task(&mut self) {
        let database = self.database.clone();
        let live_config = self.live_config.clone();
        let timer_config = live_config.timer_config.unwrap();
        let variables = live_config.variables.clone();
        let node_name = self.get_node_name().clone();
        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();
        let all_output_handle = self.get_all_output_handle().clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(format!("{}: 注册处理变量任务", node_name),
        move || {
            let strategy_id = strategy_id.clone();
            let node_id = node_id.clone();
            let node_name = node_name.clone();
            let variables = variables.clone();
            let database = database.clone();
            let all_output_handle = all_output_handle.clone();
            async move {
                Self::process_variable(
                    strategy_id, 
                    node_id,
                    node_name, 
                    variables, 
                    database, 
                    all_output_handle).await
                }
            },
            timer_config.get_millisecond()/100
        ).await;
    }

    
    pub async fn get_variable(&mut self) {
        let variables = self.live_config.variables.clone();

        for var in variables {
            let variable_type = var.variable.clone();
            match variable_type {
                SysVariable::PositionNumber => {
                    Self::get_position_number(
                        &self.database, 
                        self.get_strategy_id().clone(), 
                        self.get_node_id().clone(), 
                        self.get_node_name().clone(), 
                        var, 
                        &self.get_all_output_handle()
                    ).await.unwrap();
                }
                _ => {}
            }
        }
    }
    
    async fn process_variable(
        strategy_id: i32,
        node_id: String,
        node_name: String, 
        variables: Vec<GetVariableConfig>, 
        database: DatabaseConnection,
        output_handle: HashMap<String, NodeOutputHandle>,

    ) {
        
        for var in variables {
            let variable_type = var.variable.clone();
            match variable_type {
                SysVariable::PositionNumber => {
                    Self::get_position_number(&database, strategy_id, node_id.clone(), node_name.clone(), var, &output_handle).await;
                }
                _ => {}
            }
        }
        
        
    }

    async fn get_position_number(
        database: &DatabaseConnection, 
        strategy_id: i32,
        node_id: String,
        node_name: String,
        variable: GetVariableConfig,
        output_handle: &HashMap<String, NodeOutputHandle>,
    ) -> Result<(), String> {
        let position_numeber = StrategySysVariableQuery::get_strategy_position_number(database, strategy_id).await;
        match position_numeber {
            Ok(position_number) => {
                let variable_message = VariableMessage {
                    from_node_id: node_id.clone(),
                    from_node_name: node_name.clone(),
                    from_node_handle_id: variable.config_id.clone(), // 使用config_id作为handle_id
                    variable: variable.variable.to_string(),
                    variable_value: position_number as f64,
                    message_timestamp: get_utc8_timestamp_millis(),
                };
                let output_handle = output_handle.get(&variable.config_id).unwrap();
                output_handle.message_sender.send(NodeMessage::Variable(variable_message)).unwrap();
                Ok(())
            }
            Err(e) => {
                tracing::error!("获取持仓数量失败: {:?}", e);
                Err(e.to_string())
            }
        }
    }
}

