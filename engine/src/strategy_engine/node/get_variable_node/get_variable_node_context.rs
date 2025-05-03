
use super::super::node_context::{BaseNodeContext,NodeContext};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use types::strategy::message::NodeMessage;
use types::strategy::message::Signal;
use super::get_variable_node_types::*;



#[derive(Debug, Clone)]
pub struct GetVariableNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: Option<GetVariableNodeLiveConfig>,
    pub simulate_config: Option<GetVariableNodeSimulateConfig>,
    pub backtest_config: Option<GetVariableNodeBacktestConfig>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
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
                tracing::info!("{}: 收到信号: {:?}", self.get_node_name(), signal_message.signal);
                match signal_message.signal {
                    // 如果信号为True，则执行下单
                    Signal::True => {
                        // self.create_order().await;
                        // self.create_order2().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}

