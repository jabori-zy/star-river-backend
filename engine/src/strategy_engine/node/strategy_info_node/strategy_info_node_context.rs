
use types::market::Exchange;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use uuid::Uuid;
use types::position::PositionNumberRequest;
use super::super::node_context::{BaseNodeContext, NodeContext};
use types::strategy::message::NodeMessage;
use event_center::response_event::ResponseEvent;
use event_center::response_event::exchange_engine_response::ExchangeEngineResponse;
use types::strategy::message::Signal;
use event_center::command_event::CommandEvent;
use event_center::command_event::position_engine_command::{PositionEngineCommand, GetPositionNumberParam};




#[derive(Debug, Clone)]
pub struct StrategyInfoNodeContext {
    pub base_context: BaseNodeContext,
    pub exchange: Exchange,
    pub symbol: String,
    pub request_id: Option<Uuid>,
    pub position_number_request: PositionNumberRequest, // 订单请求
    pub position_number: i32,
}

impl StrategyInfoNodeContext {
    async fn handle_response_event(&mut self, response_event: ResponseEvent){
        let request_id= {
            match self.request_id {
                Some(id) => {
                    id
                },
                None => {
                    return;
                }
            }
        };
        match response_event {
            ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(register_exchange_success_response)) => {
                if request_id == register_exchange_success_response.response_id {
                    tracing::info!("{}: 交易所注册成功: {:?}", self.get_node_id(), register_exchange_success_response);
                    self.request_id = None;
                }
            }
            _ => {}
        }
    }

    async fn get_position_number(&self) {
        let get_position_number_params = GetPositionNumberParam {
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            position_number_request: self.position_number_request.clone(),
            sender: self.get_node_id().clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: Uuid::new_v4(),
        };
        tracing::info!("{}: 发送创建订单命令: {:?}", self.get_node_id(), get_position_number_params);
        let command_event = CommandEvent::PositionEngine(PositionEngineCommand::GetPositionNumber(get_position_number_params));
        self.get_event_publisher().publish(command_event.into()).expect("发送创建订单命令失败");
    }
}

#[async_trait]
impl NodeContext for StrategyInfoNodeContext {
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
                match signal_message.signal {
                    // 如果信号为True，则执行命令
                    Signal::True => {
                        self.get_position_number().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}

