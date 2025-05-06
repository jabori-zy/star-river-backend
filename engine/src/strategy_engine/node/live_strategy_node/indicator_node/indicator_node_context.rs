
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use async_trait::async_trait;
use event_center::Event;
use event_center::command_event::CommandEvent;
use event_center::command_event::indicator_engine_command::CalculateIndicatorParams;
use event_center::response_event::ResponseEvent;
use event_center::response_event::indicator_engine_response::IndicatorEngineResponse;
use event_center::command_event::indicator_engine_command::IndicatorEngineCommand;
use event_center::strategy_event::StrategyEvent;
use utils::get_utc8_timestamp_millis;
use types::strategy::message::{IndicatorMessage, NodeMessage};
use crate::strategy_engine::node::node_context::{BaseNodeContext,NodeContext};
use super::indicator_node_type::IndicatorNodeLiveConfig;
use types::strategy::TradeMode;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: IndicatorNodeLiveConfig,
    pub current_batch_id: Option<String>,
    pub request_id: Option<Uuid>,
}




#[async_trait]
impl NodeContext for IndicatorNodeContext {
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
            NodeMessage::KlineSeries(kline_series_message) => {
                tracing::debug!("{}: 收到K线系列消息", self.base_context.node_name);
                // 向指标引擎发送计算请求
                let request_id = Uuid::new_v4();
                let batch_id = kline_series_message.batch_id;

                
                let calculate_indicator_params = CalculateIndicatorParams {
                    exchange: kline_series_message.exchange,
                    symbol: kline_series_message.symbol,
                    interval: kline_series_message.interval,
                    indicator: self.live_config.indicator.clone(),
                    kline_series: kline_series_message.kline_series,
                    sender: self.base_context.node_id.to_string(),
                    command_timestamp: get_utc8_timestamp_millis(),
                    request_id: request_id,
                    batch_id: batch_id.clone(),
                };

                self.current_batch_id = Some(batch_id);
                self.request_id = Some(request_id);

                let event = Event::Command(CommandEvent::IndicatorEngine(IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params)));
                if let Err(e) = self.base_context.event_publisher.publish(event) {
                    tracing::error!("节点{}发送指标计算请求失败: {}", self.base_context.node_id, e);
                }
            }
            _ => {}
        }
        Ok(())
    }

}

impl IndicatorNodeContext {
    async fn handle_response_event(&self, response_event: ResponseEvent) {
        match response_event {
            ResponseEvent::IndicatorEngine(indicator_engine_response) => {
                // self.handle_indicator_engine_response(indicator_engine_response).await;
                match indicator_engine_response {
                    IndicatorEngineResponse::CalculateIndicatorFinish(calculate_indicator_response) => {
                        let (current_batch_id, request_id) = {
                            // 这里可能接收到别的节点的计算结果，而自己节点的current_batch_id和request_id为None，所以需要使用unwrap_or_default
                            let current_batch_id = self.current_batch_id.clone();
                            let request_id = self.request_id.clone();
                            if current_batch_id.is_none() || request_id.is_none() {
                                tracing::warn!("current_batch_id或request_id为None");
                                return;
                            }
                            (current_batch_id.unwrap_or_default(), request_id.unwrap_or_default())
                        };
                        let response_batch_id = calculate_indicator_response.batch_id;
                        let response_id = calculate_indicator_response.response_id;
                        // 如果请求id和批次id都匹配，则认为计算结果有效
                        if current_batch_id == response_batch_id && request_id == response_id {
                            // 计算结果有效
                            let indicator = calculate_indicator_response.indicator;
                            let indicator_value = calculate_indicator_response.value;
                            // tracing::info!("节点{}计算指标完成: {:?}", self.base_context.node_name, indicator_value);

                            let (exchange, symbol, interval) = (self.live_config.exchange.clone(), self.live_config.symbol.clone(), self.live_config.interval.clone());
                            
                            
                            let indicator_message = IndicatorMessage {
                                from_node_id: self.base_context.node_id.clone(),
                                from_node_name: self.base_context.node_name.clone(),
                                exchange: exchange,
                                symbol: symbol,
                                interval: interval,
                                indicator: indicator,
                                indicator_data: indicator_value,
                                batch_id: current_batch_id,
                                message_timestamp: get_utc8_timestamp_millis(),
                            };
                            // 获取handle的连接数
                            let default_handle_connect_count = self.base_context.output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在").connect_count;
                            // 如果连接数为0，则不发送数据
                            if default_handle_connect_count > 0 {
                                let default_output_handle = self.base_context.output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在");
                                match default_output_handle.message_sender.send(NodeMessage::Indicator(indicator_message.clone())) {
                                    Ok(_) => {
                                    // tracing::info!("节点{}发送指标数据: {:?} 发送成功, 接收者数量 = {}", state_guard.node_id, indicator_message, receiver_count);
                                }
                                Err(_) => {
                                    tracing::error!("节点{}发送指标数据失败, 接收者数量 = {}", self.base_context.node_id, default_output_handle.connect_count);
                                    }
                                }
                            } 
                            // 发送事件
                            if self.base_context.is_enable_event_publish {
                                let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Indicator(indicator_message.clone())));
                                if let Err(_) = self.base_context.event_publisher.publish(event.into()) {
                                    tracing::error!(
                                        node_id = %self.base_context.node_id,
                                        "指标节点发送数据失败"
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
}

