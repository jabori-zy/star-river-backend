
use types::market::Exchange;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use uuid::Uuid;
use super::super::node_context::{BaseNodeContext,NodeContext};
use types::strategy::message::NodeMessage;
use event_center::response_event::ResponseEvent;
use types::strategy::message::Signal;
use event_center::command_event::CommandEvent;
use event_center::command_event::order_engine_command::OrderEngineCommand;
use event_center::command_event::order_engine_command::CreateOrderParams;
use event_center::response_event::exchange_engine_response::ExchangeEngineResponse;
use event_center::command_event::BaseCommandParams;
use super::order_node_types::*;
use types::strategy::TradeMode;




#[derive(Debug, Clone)]
pub struct OrderNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: Option<OrderNodeLiveConfig>,
    pub simulate_config: Option<OrderNodeSimulateConfig>,
    pub backtest_config: Option<OrderNodeBacktestConfig>,
    pub request_id: Option<Uuid>,
}

impl OrderNodeContext {
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
            ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeResponse(register_exchange_success_response)) => {
                if request_id == register_exchange_success_response.response_id {
                    tracing::info!("{}: 交易所注册成功: {:?}", self.get_node_id(), register_exchange_success_response);
                    self.request_id = None;
                }
            }
            _ => {}
        }
    }

    async fn create_order(&self) {
        
        match self.base_context.trade_mode {
            TradeMode::Live => {
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
                let default_handle_connect_count = self.base_context.output_handle.get("order_node_output").expect("订单节点默认的消息发送器不存在").sender.receiver_count();
                if default_handle_connect_count > 0 {
                    tracing::info!("{}: 发送创建订单命令: {:?}", self.get_node_id(), create_order_params);
                    let command_event = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(create_order_params));
                    self.get_event_publisher().publish(command_event.into()).expect("发送创建订单命令失败");
                }


            }
            _ => {
                tracing::error!("{}: 暂不支持的TradeMode: {:?}", self.get_node_id(), self.base_context.trade_mode);
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
                tracing::info!("{}: 收到信号: {:?}", self.get_node_name(), signal_message.signal);
                match signal_message.signal {
                    // 如果信号为True，则执行下单
                    Signal::True => {
                        self.create_order().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

}

