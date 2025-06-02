use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use async_trait::async_trait;
use event_center::Event;
use event_center::command::Command;
use event_center::response::Response;
use event_center::response::indicator_engine_response::IndicatorEngineResponse;
use event_center::command::indicator_engine_command::{IndicatorEngineCommand, RegisterIndicatorParams};
use utils::get_utc8_timestamp_millis;
use types::strategy::node_message::{IndicatorMessage, NodeMessage};
use crate::strategy_engine::node::node_context::{LiveBaseNodeContext,LiveNodeContextTrait};
use super::indicator_node_type::IndicatorNodeLiveConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::cache::cache_key::IndicatorCacheKey;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::cache::CacheValue;
use tokio::sync::oneshot;
use event_center::response::ResponseTrait;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: LiveBaseNodeContext,
    pub live_config: IndicatorNodeLiveConfig,
    pub is_registered: Arc<RwLock<bool>>, // 是否已经注册指标
}




#[async_trait]
impl LiveNodeContextTrait for IndicatorNodeContext {
    fn clone_box(&self) -> Box<dyn LiveNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &LiveBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut LiveBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("indicator_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {

        // if let Event::Response(response_event) = event {
        //     match response_event {
        //         ResponseEvent::IndicatorEngine(indicator_engine_response) => {
        //             match indicator_engine_response {
        //                 // 注册指标完成
        //                 IndicatorEngineResponse::RegisterIndicatorResponse(register_indicator_response) => {
        //             // 检查请求id是否在request_ids中
        //             let is_contain = self.check_and_remove_request_id(register_indicator_response.response_id).await;
        //             // 如果请求id在request_ids中，处理响应事件
        //             if is_contain {                        // 判断状态码
        //                 if register_indicator_response.code == 0 {
        //                     // 设置为已注册
        //                     *self.is_registered.write().await = true;
        //                 } else {
        //                     tracing::error!("节点{}收到指标注册失败事件: {:?}", self.base_context.node_id, register_indicator_response);
        //                 }
        //                     }
        //                 }
        //             }
        //         }
        //         ResponseEvent::CacheEngine(cache_engine_response) => {
        //             match cache_engine_response {
        //                 CacheEngineResponse::GetCacheData(get_cache_data_response) => {
        //                     // 处理缓存响应事件
        //                     let is_contain = self.check_and_remove_request_id(get_cache_data_response.response_id).await;
        //                     if is_contain {
        //                         // 判断状态码
        //                         if get_cache_data_response.code == 0 {
        //                             // 发送指标message
        //                             self.publish_indicator_message(get_cache_data_response.cache_data).await;
                                    
        //                         }
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         _ => {}
        //     }
        // }


        Ok(())
    }

    
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::KlineSeries(_) => {
                // 接收到k线数据， 向缓存引擎请求指标数据
                let indicator_cache_key = IndicatorCacheKey::new(
                    self.live_config.exchange.clone(), 
                    self.live_config.symbol.clone(), 
                    self.live_config.interval.clone(), 
                    self.live_config.indicator_config.clone());
                let response = self.get_indicator_cache(indicator_cache_key).await;
                if let Ok(response) = response {
                    if response.code() == 0 {
                        let cache_engine_response = CacheEngineResponse::try_from(response).unwrap();
                        match cache_engine_response {
                            CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                                let indicator_series = get_cache_data_response.cache_data.clone();
                                // 发送指标数据
                                let indicator_message = IndicatorMessage {
                                    from_node_id: self.base_context.node_id.clone(),
                                    from_node_name: self.base_context.node_name.clone(),
                                    exchange: self.live_config.exchange.clone(),
                                    symbol: self.live_config.symbol.clone(),
                                    interval: self.live_config.interval.clone(),
                                    indicator_config: self.live_config.indicator_config.clone(),
                                    indicator_series: indicator_series,
                                    message_timestamp: get_utc8_timestamp_millis(),
                                };
                                tracing::info!("节点{}收到指标缓存数据: {:?}", self.base_context.node_id, indicator_message);
                                // 发送指标message
                                let handle = self.get_default_output_handle();
                                handle.send(NodeMessage::Indicator(indicator_message)).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

}

impl IndicatorNodeContext {

    async fn get_indicator_cache(&self, indicator_cache_key: IndicatorCacheKey) -> Result<Response, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            cache_key: indicator_cache_key.into(),
            index: None,
            limit: Some(2),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };
        let get_cache_command = CacheEngineCommand::GetCache(params);
        self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

        // 等待响应
        let get_cache_response = resp_rx.await.unwrap();
        tracing::info!("节点{}收到指标缓存数据: {:?}", self.base_context.node_id, get_cache_response);
        Ok(get_cache_response)
    }


    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator(&self) -> Result<Response, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let register_indicator_params = RegisterIndicatorParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            exchange: self.live_config.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            indicator_config: self.live_config.indicator_config.clone(),
            sender: self.base_context.node_id.to_string(),
            command_timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };
        let register_indicator_command = Command::IndicatorEngine(IndicatorEngineCommand::RegisterIndicator(register_indicator_params));
        self.get_command_publisher().send(register_indicator_command).await.unwrap();

        // 等待响应
        let register_indicator_response = resp_rx.await.unwrap();
        Ok(register_indicator_response)
    }
    
}

