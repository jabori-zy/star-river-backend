use super::CacheEngineContext;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::communication::Command;
use event_center::communication::engine::cache_engine::CacheEngineCommand;
use event_center::communication::engine::cache_engine::*;
use event_center::communication::engine::{EngineCommand, EngineResponse};
use event_center::event::Event;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use star_river_core::cache::CacheItem;

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
                    CacheEngineCommand::AddKey(cmd) => {
                        self.add_key(cmd.key.clone(), cmd.max_size, cmd.duration)
                            .await
                            .unwrap();
                        let payload = AddKeyRespPayload::new(cmd.key.clone());
                        let response = AddKeyResponse::success(Some(payload));
                        cmd.respond(response);
                    }

                    // 处理获取缓存数据命令
                    CacheEngineCommand::GetCache(cmd) => {
                        let data = self.get_cache(&cmd.key, cmd.index, cmd.limit).await;
                        let payload = GetCacheRespPayload::new(data);
                        let response = GetCacheResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    CacheEngineCommand::GetCacheMulti(cmd) => {
                        let multi_data = self.get_cache_multi(&cmd.keys, cmd.index, cmd.limit).await;

                        let multi_data_result = multi_data
                            .into_iter()
                            .map(|(cache_key, data)| {
                                (
                                    cache_key.get_key_str(),
                                    data.into_iter().map(|cache_value| cache_value.to_list()).collect(),
                                )
                            })
                            .collect();
                        let payload = GetCacheMultiRespPayload::new(multi_data_result);
                        let response = GetCacheMultiResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    CacheEngineCommand::GetCacheLengthMulti(cmd) => {
                        let mut length_result = HashMap::new();
                        for key in cmd.keys.iter() {
                            length_result.insert(key.clone(), self.get_cache_length(key).await);
                        }

                        let payload = GetCacheLengthMultiRespPayload::new(cmd.keys.clone(), length_result);
                        let response = GetCacheLengthMultiResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    // 更新缓存
                    CacheEngineCommand::UpdateCache(cmd) => {
                        self.update_cache(cmd.key.clone(), cmd.value.as_ref().clone()).await;
                        let payload = UpdateCacheRespPayload::new(cmd.key.clone());
                        let response = UpdateCacheResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    // 清空缓存
                    CacheEngineCommand::ClearCache(cmd) => {
                        self.clear_cache(cmd.key.clone()).await;
                        let payload = ClearCacheRespPayload::new(cmd.key.clone());
                        let response = ClearCacheResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
