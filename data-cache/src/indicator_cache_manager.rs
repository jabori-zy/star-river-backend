use crate::{CacheManager, CacheEntry};
use std::collections::VecDeque;
use event_center::response_event::{CacheEngineResponse, ResponseEvent};
use types::market::Exchange;
use utils::get_utc8_timestamp_millis;
use types::indicator::IndicatorData;
use types::cache::IndicatorCacheKey;
use types::market::KlineInterval;
use event_center::command_event::GetSubscribedIndicatorParams;
use event_center::response_event::{SubscribedIndicatorResponse};
use event_center::EventPublisher;

// 修改 CacheEntry 以接受实现了 Indicator 特征的类型
impl CacheEntry<IndicatorCacheKey, Box<dyn IndicatorData>> {
    pub fn initialize(&mut self, data: VecDeque<Box<dyn IndicatorData>>) {
        self.data = data;
        self.is_fresh = true;
        self.updated_at = get_utc8_timestamp_millis();
    }
    pub fn insert_or_update(&mut self, indicator_data: Box<dyn IndicatorData>) {}
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
    //     if self.data.back().unwrap().timestamp == indicator_data.timestamp {
    //         self.data.pop_back();
    //         self.data.push_back(indicator_data);
    //         self.is_fresh = true;
    //         self.updated_at = get_utc8_timestamp();
    //     } else {
    //         self.data.push_back(indicator_data);
    //         self.is_fresh = true;
    //         self.updated_at = get_utc8_timestamp();
    //     }

    // }
}


impl CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>> {
    pub async fn initialize_indicator_cache(&mut self, indicator_cache_key: IndicatorCacheKey) {
        // 判断key是否已存在，如果不存在，则添加订阅
        if !self.cache.contains_key(&indicator_cache_key) {
            self.subscribe(indicator_cache_key.clone());
        }
        
    }

    pub fn is_exists(&self, indicator_cache_key: IndicatorCacheKey) -> bool {
        self.cache.contains_key(&indicator_cache_key)
    }

    pub fn get_key_list(&self) -> Vec<IndicatorCacheKey> {
        self.cache.keys().cloned().collect()
    }

    // 获取k线系列需要计算的指标
    pub fn get_subscribed_indicator(&self, params: GetSubscribedIndicatorParams, event_publisher: EventPublisher) {
        tracing::debug!("接收到的参数: {:?}", params);
        let mut sub_indicator_key_list = Vec::new();
        for (key, _) in self.cache.iter() {
            // 如果key的symbol和interval和exchange都匹配，则加入sub_indicator_key_list
            if key.symbol == params.symbol && key.interval == params.interval && key.exchange == params.exchange {
                sub_indicator_key_list.push(key.clone());
            }
        }

        
        
        let response_id = params.request_id;
        let response = CacheEngineResponse::SubscribedIndicator(SubscribedIndicatorResponse {
            indicator_cache_key_list: sub_indicator_key_list,
            response_timestamp: get_utc8_timestamp_millis(),
            response_id: response_id,
        });
        let response_event = ResponseEvent::CacheEngine(response);
        // tracing::debug!("返回值发送成功 {:?}", response_event);
        event_publisher.publish(response_event.into()).expect("返回值发送失败");

    }

}
