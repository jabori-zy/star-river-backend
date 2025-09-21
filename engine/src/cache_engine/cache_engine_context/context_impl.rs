use super::CacheEngineContext;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::communication::engine::cache_engine::CacheEngineCommand;
use event_center::communication::engine::cache_engine::*;
use event_center::communication::engine::{EngineCommand, EngineResponse};
use event_center::event::Event;
use std::any::Any;
use std::collections::HashMap;


#[async_trait]
impl EngineContext for CacheEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Exchange(exchange_event) => {
                self.handle_exchange_event(exchange_event).await;
            }
            Event::Indicator(indicator_event) => {
                self.handle_indicator_event(indicator_event).await;
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::CacheEngine(command) => {
                match command {
                    // 添加缓存
                    CacheEngineCommand::AddCacheKey(params) => {
                        self.add_key(params.key.clone(), params.max_size, params.duration)
                            .await
                            .unwrap();
                        let response = AddCacheKeyResponse::success(params.key);
                        let response_event =
                            EngineResponse::CacheEngine(CacheEngineResponse::AddCacheKey(response));

                        params.responder.send(response_event.into()).unwrap();
                    }

                    // 处理获取缓存数据命令
                    CacheEngineCommand::GetCache(params) => {
                        let data = self
                            .get_cache(&params.key, params.index, params.limit)
                            .await;
                        let response = GetCacheDataResponse::success(params.key, data);
                        let response = CacheEngineResponse::GetCacheData(response);
                        params.responder.send(response.into()).unwrap();
                    }
                    CacheEngineCommand::GetCacheMulti(params) => {
                        let multi_data = self
                            .get_cache_multi(&params.keys, params.index, params.limit)
                            .await;
                        let response = GetCacheDataMultiResponse::success(multi_data);
                        let response = CacheEngineResponse::GetCacheDataMulti(response);
                        params.responder.send(response.into()).unwrap();
                    }
                    CacheEngineCommand::GetCacheLengthMulti(params) => {
                        let mut length_result = HashMap::new();
                        for key in params.keys.iter() {
                            length_result.insert(key.clone(), self.get_cache_length(key).await);
                        }

                        let get_cache_length_multi_response =
                            GetCacheLengthMultiResponse::success(length_result);
                        let response = CacheEngineResponse::GetCacheLengthMulti(
                            get_cache_length_multi_response,
                        );
                        params.responder.send(response.into()).unwrap();
                    }
                    // 更新缓存
                    CacheEngineCommand::UpdateCache(params) => {
                        self.update_cache(params.key.clone(), params.cache_value).await;
                        let response: CacheEngineResponse = UpdateCacheResponse::success(params.key).into();
                        params.responder.send(response.into()).unwrap();
                    }
                    // 清空缓存
                    CacheEngineCommand::ClearCache(params) => {
                        self.clear_cache(params.key.clone()).await;
                        let response: CacheEngineResponse = ClearCacheResponse::success(params.key).into();
                        params.responder.send(response.into()).unwrap();
                    }
                    _ => {}
                    
                }
            }
            _ => {}
        }
    }
}