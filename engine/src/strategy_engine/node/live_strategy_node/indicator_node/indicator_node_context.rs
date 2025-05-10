use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use async_trait::async_trait;
use event_center::Event;
use event_center::command_event::CommandEvent;
use event_center::response_event::ResponseEvent;
use event_center::response_event::indicator_engine_response::IndicatorEngineResponse;
use event_center::command_event::indicator_engine_command::{IndicatorEngineCommand, RegisterIndicatorParams};
use utils::get_utc8_timestamp_millis;
use types::strategy::node_message::{IndicatorMessage, NodeMessage};
use crate::strategy_engine::node::node_context::{BaseNodeContext,NodeContext};
use super::indicator_node_type::IndicatorNodeLiveConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use event_center::command_event::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::cache::cache_key::IndicatorCacheKey;
use event_center::response_event::cache_engine_response::CacheEngineResponse;
use types::cache::CacheValue;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: IndicatorNodeLiveConfig,
    pub current_batch_id: Option<String>,
    pub is_registered: Arc<RwLock<bool>>, // 是否已经注册指标
    pub request_ids: Arc<Mutex<Vec<Uuid>>>,
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

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("indicator_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {

        if let Event::Response(response_event) = event {
            match response_event {
                ResponseEvent::IndicatorEngine(indicator_engine_response) => {
                    match indicator_engine_response {
                        // 注册指标完成
                        IndicatorEngineResponse::RegisterIndicatorResponse(register_indicator_response) => {
                    // 检查请求id是否在request_ids中
                    let is_contain = self.check_and_remove_request_id(register_indicator_response.response_id).await;
                    // 如果请求id在request_ids中，处理响应事件
                    if is_contain {                        // 判断状态码
                        if register_indicator_response.code == 0 {
                            // 设置为已注册
                            *self.is_registered.write().await = true;
                        } else {
                            tracing::error!("节点{}收到指标注册失败事件: {:?}", self.base_context.node_id, register_indicator_response);
                        }
                            }
                        }
                    }
                }
                ResponseEvent::CacheEngine(cache_engine_response) => {
                    match cache_engine_response {
                        CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                            // 处理缓存响应事件
                            let is_contain = self.check_and_remove_request_id(get_cache_data_response.response_id).await;
                            if is_contain {
                                // 判断状态码
                                if get_cache_data_response.code == 0 {
                                    // 发送指标message
                                    self.publish_indicator_message(get_cache_data_response.cache_data).await;
                                    
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }


        Ok(())
    }

    
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::KlineSeries(_) => {
                // 接收到k线数据， 向缓存引擎请求指标数据
                let request_id = Uuid::new_v4();
                let indicator_cache_key = IndicatorCacheKey::new(
                    self.live_config.exchange.clone(), 
                    self.live_config.symbol.clone(), 
                    self.live_config.interval.clone(), 
                    self.live_config.indicator_config.clone());
                self.publish_get_indicator_cache_command(indicator_cache_key, request_id).await;
            }
            _ => {}
        }
        Ok(())
    }

}

impl IndicatorNodeContext {
    async fn check_and_remove_request_id(&mut self, request_id: Uuid) -> bool {
        let mut request_ids = self.request_ids.lock().await;
        if let Some(index) = request_ids.iter().position(|id| *id == request_id) {
            request_ids.remove(index);
            true
        } else {
            false
        }
    }

    async fn publish_get_indicator_cache_command(
        &self, 
        indicator_cache_key: IndicatorCacheKey, request_id: Uuid
    ) {
        let params = GetCacheParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            cache_key: indicator_cache_key.into(),
            limit: Some(2),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };
        self.request_ids.lock().await.push(request_id);
        let event = Event::Command(CommandEvent::CacheEngine(CacheEngineCommand::GetCache(params)));
        if let Err(e) = self.base_context.event_publisher.publish(event) {
            tracing::error!("节点{}发送指标缓存请求失败: {}", self.base_context.node_id, e);
        }
    }

    async fn publish_indicator_message(&self, indicator_series: Vec<Arc<CacheValue>>) {
        let indicator_message = IndicatorMessage {
            from_node_id: self.base_context.node_id.clone(),
            from_node_name: self.base_context.node_name.clone(),
            exchange: self.live_config.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            indicator: self.live_config.indicator_config.clone(),
            indicator_series: indicator_series,
            message_timestamp: get_utc8_timestamp_millis(),
        };
        tracing::info!("节点{}收到指标缓存数据: {:?}", self.base_context.node_id, indicator_message);
        // 发送指标message
        let handle = self.get_default_output_handle();
        handle.send(NodeMessage::Indicator(indicator_message)).unwrap();
    }


    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator(&self) {
        let request_id = Uuid::new_v4();
        let register_indicator_params = RegisterIndicatorParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            exchange: self.live_config.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            indicator_config: self.live_config.indicator_config.clone(),
            sender: self.base_context.node_id.to_string(),
            command_timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };
        // 添加请求id
        self.request_ids.lock().await.push(request_id);

        let event = Event::Command(CommandEvent::IndicatorEngine(IndicatorEngineCommand::RegisterIndicator(register_indicator_params)));
        if let Err(e) = self.base_context.event_publisher.publish(event) {
            tracing::error!("节点{}发送指标注册请求失败: {}", self.base_context.node_id, e);
        }
    }
    
}

