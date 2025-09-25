use super::CacheEngineContext;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::communication::Command;
use event_center::communication::engine::cache_engine::CacheEngineCommand;
use event_center::communication::engine::cache_engine::*;
use event_center::communication::engine::EngineCommand;
use event_center::event::Event;
use star_river_core::key::Key;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use star_river_core::key::KeyTrait;
use star_river_core::market::QuantData;

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
                    // 添加缓存键
                    CacheEngineCommand::AddKlineKey(cmd) => self.handle_add_kline_key(cmd).await,
                    // 获取K线缓存
                    CacheEngineCommand::GetKlineCache(cmd) => self.handle_get_kline_cache(cmd).await,
                    // 批量获取K线缓存
                    CacheEngineCommand::GetKlineCacheMulti(cmd) => self.handle_get_kline_cache_multi(cmd).await,
                    // 获取K线缓存长度
                    CacheEngineCommand::GetKlineCacheLengthMulti(cmd) => self.handle_get_kline_cache_length_multi(cmd).await,
                    // 更新缓存
                    CacheEngineCommand::UpdateKlineCache(cmd) => self.handle_update_kline_cache(cmd).await,
                    // 清空缓存
                    CacheEngineCommand::ClearKlineCache(cmd) => self.handle_clear_kline_cache(cmd).await,
                    // 添加指标缓存键
                    CacheEngineCommand::AddIndicatorKey(cmd) => self.handle_add_indicator_key(cmd).await,
                    // 获取指标缓存
                    CacheEngineCommand::GetIndicatorCache(cmd) => self.handle_get_indicator_cache(cmd).await,
                    // 批量获取指标缓存
                    CacheEngineCommand::GetIndicatorCacheMulti(cmd) => self.handle_get_indicator_cache_multi(cmd).await,
                    // 获取指标缓存长度
                    CacheEngineCommand::GetIndicatorCacheLengthMulti(cmd) => self.handle_get_indicator_cache_length_multi(cmd).await,
                    // 更新指标缓存
                    CacheEngineCommand::UpdateIndicatorCache(cmd) => self.handle_update_indicator_cache(cmd).await,
                    // 清空指标缓存
                    CacheEngineCommand::ClearIndicatorCache(cmd) => self.handle_clear_indicator_cache(cmd).await,
                    _ => {}
                    
                }
            }
            _ => {}
        }
    }
}
