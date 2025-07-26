use crate::cache::Key;
use std::collections::VecDeque;
use std::time::Duration;
use utils::get_utc8_timestamp_millis;
use crate::cache::*;
use deepsize::DeepSizeOf;
use std::sync::Arc;

// #[derive(Debug, Clone)]
// pub struct KlineCacheEntry {
//     pub key: BackKlineKey, // 缓存键
//     pub data: VecDeque<Arc<CacheValue>>, // 缓存数据
//     pub create_time: i64, // 创建时间
//     pub update_time: i64, // 更新时间
//     pub max_size: u32, // 最大大小
//     pub is_fresh: bool, // 是否新鲜
//     pub ttl: Duration, // 过期时间 time to live
//
// }
//
// // pub access_stats: AccessStats, // 访问统计
// // pub config: CacheConfig,       // 缓存配置
// // }
//
// // // 访问统计
// // #[derive(Debug, Clone)]
// // pub struct AccessStats {
// // pub create_time: i64,          // 创建时间
// // pub update_time: i64,          // 更新时间
// // pub last_access_time: i64,     // 最后访问时间
// // pub access_count: u64,         // 访问计数
// // pub hit_count: u64,            // 命中计数
// // }
//
// // // 缓存配置
// // #[derive(Debug, Clone)]
// // pub struct CacheConfig {
// // pub max_size: u32,             // 最大大小
// // pub ttl: Duration,             // 过期时间
// // pub is_fresh: bool,            // 是否新鲜
// // pub compression_enabled: bool, // 是否启用压缩
// // pub memory_limit: Option<u32>, // 内存限制(字节)
// // }
//
//
// impl From<KlineCacheEntry> for CacheEntry {
//     fn from(entry: KlineCacheEntry) -> Self {
//         CacheEntry::Kline(entry)
//     }
// }
//
// impl KlineCacheEntry {
//     pub fn new(key: KlineKey, max_size: u32, ttl: Duration) -> Self {
//         Self {
//             key,
//             data: VecDeque::new(),
//             create_time: get_utc8_timestamp_millis(),
//             update_time: get_utc8_timestamp_millis(),
//             max_size,
//             is_fresh: false,
//             ttl,
//         }
//     }
// }
//
// impl CacheEntryTrait for KlineCacheEntry {
//     fn initialize(&mut self, data: Vec<CacheValue>) {
//         self.data = data.into_iter().map(|value| value.into()).collect();
//         self.is_fresh = true;
//         self.update_time = get_utc8_timestamp_millis();
//     }
//
//     fn update(&mut self, cache_value: CacheValue) {
//         // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
//         if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
//             self.data.pop_back();
//             self.data.push_back(cache_value.into());
//         } else {
//             // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
//             // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
//             if self.data.len() >= self.max_size as usize {
//                 self.data.pop_front();
//             }
//             self.data.push_back(cache_value.into());
//         }
//     }
//
//     fn get_key(&self) -> Key {
//         Key::Kline(self.key.clone())
//     }
//
//     fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
//         self.data.iter().cloned().collect()
//     }
//
//     // 获取缓存数据
//     // index: 缓存索引
//     // limit: 缓存数量(倒着取)
//     fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
//         // 如果limit为None，则返回index之前的所有数据
//         if limit.is_none() {
//             return self.get_all_cache_data();
//         }
//
//         let limit = limit.unwrap();
//
//         // 处理index参数
//         if let Some(idx) = index {
//             // 确保索引在有效范围内
//             if idx as usize >= self.data.len() {
//                 return Vec::new();
//             }
//
//             // 计算从索引开始向前取limit个元素
//             let end = idx as usize + 1; // 索引位置加1（包含索引位置）
//             let start = if limit as usize >= end {
//                 0 // 如果limit大于等于end，就取从0到end的所有元素
//             } else {
//                 end - limit as usize // 否则取索引前的limit个元素
//             };
//
//             return self.data.range(start..end).cloned().collect();
//         } else {
//             // 没有指定index，使用原来的逻辑，从后往前取limit条数据
//             // 如果limit大于等于数据长度，直接克隆并返回所有数据
//             if limit as usize >= self.data.len() {
//                 return self.get_all_cache_data();
//             }
//
//             // 从后往前取limit条数据
//             let start = self.data.len().saturating_sub(limit as usize);
//             return self.data.range(start..).cloned().collect();
//         }
//     }
//
//     fn get_create_time(&self) -> i64 {
//         self.create_time
//     }
//
//     fn get_update_time(&self) -> i64 {
//         self.update_time
//     }
//
//     fn get_max_size(&self) -> u32 {
//         self.max_size
//     }
//
//     fn get_is_fresh(&self) -> bool {
//         self.is_fresh
//     }
//
//     fn get_ttl(&self) -> Duration {
//         self.ttl
//     }
//
//     fn get_length(&self) -> u32 {
//         self.data.len() as u32
//     }
//
//     fn get_memory_size(&self) -> u32 {
//         self.data.iter().map(|value| value.deep_size_of() as u32).sum()
//     }
// }
//
//
// #[derive(Debug, Clone)]
// pub struct IndicatorCacheEntry {
//     pub key: IndicatorKey, // 缓存键
//     pub data: VecDeque<Arc<CacheValue>>, // 缓存数据
//     pub create_time: i64, // 创建时间
//     pub update_time: i64, // 更新时间
//     pub max_size: u32, // 最大大小
//     pub is_fresh: bool, // 是否新鲜
//     pub ttl: Duration, // 过期时间 time to live
// }
//
// impl From<IndicatorCacheEntry> for CacheEntry {
//     fn from(entry: IndicatorCacheEntry) -> Self {
//         CacheEntry::Indicator(entry)
//     }
// }
//
//
// impl IndicatorCacheEntry {
//     pub fn new(key: IndicatorKey, max_size: u32, ttl: Duration) -> Self {
//         Self {
//             key,
//             data: VecDeque::new(),
//             create_time: get_utc8_timestamp_millis(),
//             update_time: get_utc8_timestamp_millis(),
//             max_size,
//             is_fresh: false,
//             ttl,
//         }
//     }
//
// }
//
// impl CacheEntryTrait for IndicatorCacheEntry {
//
//     fn initialize(&mut self, data: Vec<CacheValue>) {
//         self.data = data.into_iter().map(|value| value.into()).collect();
//         self.is_fresh = true;
//         self.update_time = get_utc8_timestamp_millis();
//     }
//
//     fn update(&mut self, cache_value: CacheValue) {
//         // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
//         if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
//             self.data.pop_back();
//             self.data.push_back(cache_value.into());
//         } else {
//             // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
//             // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
//             if self.data.len() >= self.max_size as usize {
//                 self.data.pop_front();
//             }
//             self.data.push_back(cache_value.into());
//         }
//     }
//
//     fn get_key(&self) -> Key {
//         Key::Indicator(self.key.clone())
//     }
//
//     fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
//         self.data.iter().map(|value| value.clone()).collect()
//     }
//
//     fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
//         // 如果limit为None，则返回所有数据
//         if limit.is_none() {
//             return self.get_all_cache_data();
//         }
//
//         let limit = limit.unwrap();
//
//         // 处理index参数
//         if let Some(idx) = index {
//             // 确保索引在有效范围内
//             if idx as usize >= self.data.len() {
//                 return Vec::new();
//             }
//
//             // 计算从索引开始向前取limit个元素
//             let end = idx as usize + 1; // 索引位置加1（包含索引位置）
//             let start = if limit as usize >= end {
//                 0 // 如果limit大于等于end，就取从0到end的所有元素
//             } else {
//                 end - limit as usize // 否则取索引前的limit个元素
//             };
//
//             return self.data.range(start..end).cloned().collect();
//         } else {
//             // 没有指定index，使用原来的逻辑，从后往前取limit条数据
//             // 如果limit大于等于数据长度，直接克隆并返回所有数据
//             if limit as usize >= self.data.len() {
//                 return self.get_all_cache_data();
//             }
//
//             // 从后往前取limit条数据
//             let start = self.data.len().saturating_sub(limit as usize);
//             return self.data.range(start..).cloned().collect();
//         }
//     }
//
//     fn get_create_time(&self) -> i64 {
//         self.create_time
//     }
//
//     fn get_update_time(&self) -> i64 {
//         self.update_time
//     }
//
//     fn get_max_size(&self) -> u32 {
//         self.max_size
//     }
//
//     fn get_is_fresh(&self) -> bool {
//         self.is_fresh
//     }
//
//     fn get_ttl(&self) -> Duration {
//         self.ttl
//     }
//
//     fn get_length(&self) -> u32 {
//         self.data.len() as u32
//     }
//
//     fn get_memory_size(&self) -> u32 {
//         self.data.iter().map(|value| value.deep_size_of() as u32).sum()
//     }
// }


// #[derive(Debug, Clone)]
// pub struct HistoryKlineCacheEntry {
//     pub key: HistoryKlineCacheKey, // 缓存键
//     pub data: VecDeque<Arc<CacheValue>>, // 缓存数据
//     pub create_time: i64, // 创建时间
//     pub update_time: i64, // 更新时间
//     pub max_size: u32, // 最大大小
//     pub is_fresh: bool, // 是否新鲜
//     pub ttl: Duration, // 过期时间 time to live
// }


// impl From<HistoryKlineCacheEntry> for CacheEntry {
//     fn from(entry: HistoryKlineCacheEntry) -> Self {
//         CacheEntry::HistoryKline(entry)
//     }
// }


// impl HistoryKlineCacheEntry {
//     pub fn new(key: HistoryKlineCacheKey, max_size: u32, ttl: Duration) -> Self {
//         Self {
//             key,
//             data: VecDeque::new(),
//             create_time: get_utc8_timestamp_millis(),
//             update_time: get_utc8_timestamp_millis(),
//             max_size,
//             is_fresh: false,
//             ttl,
//         }
//     }
// }

// impl CacheEntryTrait for HistoryKlineCacheEntry {
//     fn initialize(&mut self, data: Vec<CacheValue>) {
//         self.data = data.into_iter().map(|value| value.into()).collect();
//         self.is_fresh = true;
//         self.update_time = get_utc8_timestamp_millis();
//     }

//     fn update(&mut self, cache_value: CacheValue) {
//         // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
//         if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
//             self.data.pop_back();
//             self.data.push_back(cache_value.into());
//         } else {
//             // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
//             // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
//             if self.data.len() >= self.max_size as usize {
//                 self.data.pop_front();
//             }
//             self.data.push_back(cache_value.into());
//         }
//     }

//     fn get_key(&self) -> CacheKey {
//         CacheKey::HistoryKline(self.key.clone())
//     }

//     fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
//         self.data.iter().map(|value| value.clone()).collect()
//     }

//     fn get_cache_data(&self, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
//         // 如果limit为None，则返回所有数据
//         if limit.is_none() {
//             return self.get_all_cache_data();
//         }

//         // 如果limit大于等于数据长度，直接克隆并返回所有数据
//         let limit = limit.unwrap();
//         if limit as usize >= self.data.len() {
//             return self.get_all_cache_data();
//         }
        
//         // 从后往前取limit条数据
//         let start = self.data.len().saturating_sub(limit as usize);
//         self.data.range(start..).cloned().collect()
//     }

//     fn get_create_time(&self) -> i64 {
//         self.create_time
//     }

//     fn get_update_time(&self) -> i64 {
//         self.update_time
//     }

//     fn get_max_size(&self) -> u32 {
//         self.max_size
//     }

//     fn get_is_fresh(&self) -> bool {
//         self.is_fresh
//     }

//     fn get_ttl(&self) -> Duration {
//         self.ttl
//     }

//     fn get_length(&self) -> u32 {
//         self.data.len() as u32
//     }

//     fn get_memory_size(&self) -> u32 {
//         self.data.iter().map(|value| value.deep_size_of() as u32).sum()
//     }
// }





// #[derive(Debug, Clone)]
// pub struct HistoryIndicatorCacheEntry {
//     pub key: HistoryIndicatorCacheKey, // 缓存键
//     pub data: VecDeque<Arc<CacheValue>>, // 缓存数据
//     pub create_time: i64, // 创建时间
//     pub update_time: i64, // 更新时间
//     pub max_size: u32, // 最大大小
//     pub is_fresh: bool, // 是否新鲜
//     pub ttl: Duration, // 过期时间 time to live
// }

// impl From<HistoryIndicatorCacheEntry> for CacheEntry {
//     fn from(entry: HistoryIndicatorCacheEntry) -> Self {
//         CacheEntry::HistoryIndicator(entry)
//     }
// }


// impl HistoryIndicatorCacheEntry {
//     pub fn new(key: HistoryIndicatorCacheKey, max_size: u32, ttl: Duration) -> Self {
//         Self {
//             key,
//             data: VecDeque::new(),
//             create_time: get_utc8_timestamp_millis(),
//             update_time: get_utc8_timestamp_millis(),
//             max_size,
//             is_fresh: false,
//             ttl,
//         }
//     }
    
// }

// impl CacheEntryTrait for HistoryIndicatorCacheEntry {

//     fn initialize(&mut self, data: Vec<CacheValue>) {
//         self.data = data.into_iter().map(|value| value.into()).collect();
//         self.is_fresh = true;
//         self.update_time = get_utc8_timestamp_millis();
//     }

//     fn update(&mut self, cache_value: CacheValue) {
//         // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
//         if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
//             self.data.pop_back();
//             self.data.push_back(cache_value.into());
//         } else {
//             // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
//             // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
//             if self.data.len() >= self.max_size as usize {
//                 self.data.pop_front();
//             }
//             self.data.push_back(cache_value.into());
//         }
//     }

//     fn get_key(&self) -> CacheKey {
//         CacheKey::HistoryIndicator(self.key.clone())
//     }

//     fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
//         self.data.iter().map(|value| value.clone()).collect()
//     }

//     fn get_cache_data(&self, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
//         // 如果limit为None，则返回所有数据
//         if limit.is_none() {
//             return self.get_all_cache_data();
//         }

//         // 如果limit大于等于数据长度，直接克隆并返回所有数据
//         let limit = limit.unwrap();
//         if limit as usize >= self.data.len() {
//             return self.get_all_cache_data();
//         }
        
//         // 从后往前取limit条数据
//         let start = self.data.len().saturating_sub(limit as usize);
//         self.data.range(start..).cloned().collect()
//     }

//     fn get_create_time(&self) -> i64 {
//         self.create_time
//     }

//     fn get_update_time(&self) -> i64 {
//         self.update_time
//     }

//     fn get_max_size(&self) -> u32 {
//         self.max_size
//     }

//     fn get_is_fresh(&self) -> bool {
//         self.is_fresh
//     }

//     fn get_ttl(&self) -> Duration {
//         self.ttl
//     }

//     fn get_length(&self) -> u32 {
//         self.data.len() as u32
//     }

//     fn get_memory_size(&self) -> u32 {
//         self.data.iter().map(|value| value.deep_size_of() as u32).sum()
//     }
// }



#[derive(Debug, Clone)]
pub struct GenericCacheEntry<K: Clone + Debug + Into<Key>> {
    pub key: K,
    pub data: VecDeque<Arc<CacheValue>>, 
    pub create_time: i64,
    pub update_time: i64,
    pub max_size: Option<u32>, // 如果为None，则表示无限缓存
    pub is_fresh: bool, 
    pub ttl: Duration,
}



impl<K: Clone + Debug + Into<Key>> CacheEntryTrait for GenericCacheEntry<K> {
    fn initialize(&mut self, data: Vec<CacheValue>) {
        self.data = data.into_iter().map(|value| value.into()).collect();
        self.is_fresh = true;
        self.update_time = get_utc8_timestamp_millis();
    }

    fn update(&mut self, cache_value: CacheValue) {
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
        if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
            self.data.pop_back();
            self.data.push_back(cache_value.into());
        } else {
            // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
            // 如果max_size为None，则无限缓存
            if self.max_size.is_none() {
                self.data.push_back(cache_value.into());
            } else {
                // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
                if self.data.len() >= self.max_size.unwrap() as usize {
                    self.data.pop_front();
                }
                self.data.push_back(cache_value.into());
            }
        }
    }

    fn get_key(&self) -> Key {
        self.key.clone().into()
    }

    fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
        self.data.iter().cloned().collect()
    }

    fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        // 如果limit和index都为None，则返回所有数据
        if limit.is_none() && index.is_none() {
            return self.get_all_cache_data();
        }

        // 如果limit为None，设置为数据总长度以便后续处理
        let limit = if let Some(limit) = limit {
            limit
        } else {
            self.data.len() as u32
        };
        
        // 处理index参数
        if let Some(idx) = index {
            // 确保索引在有效范围内
            if idx as usize >= self.data.len() {
                return Vec::new();
            }
            
            // 计算从索引开始向前取limit个元素
            let end = idx as usize + 1; // 索引位置加1（包含索引位置）
            let start = if limit as usize >= end {
                0 // 如果limit大于等于end，就取从0到end的所有元素
            } else {
                end - limit as usize // 否则取索引前的limit个元素
            };
            
            return self.data.range(start..end).cloned().collect();
        } else {
            // 没有指定index，使用原来的逻辑，从后往前取limit条数据
            // 如果limit大于等于数据长度，直接克隆并返回所有数据
            if limit as usize >= self.data.len() {
                return self.get_all_cache_data();
            }
            
            // 从后往前取limit条数据
            let start = self.data.len().saturating_sub(limit as usize);
            return self.data.range(start..).cloned().collect();
        }
    }

    fn get_create_time(&self) -> i64 {
        self.create_time
    }

    fn get_update_time(&self) -> i64 {
        self.update_time
    }

    fn get_max_size(&self) -> u32 {
        if self.max_size.is_none() {
            return u32::MAX;
        }
        self.max_size.unwrap()
    }

    fn get_is_fresh(&self) -> bool {
        self.is_fresh
    }

    fn get_ttl(&self) -> Duration {
        self.ttl
    }

    fn get_length(&self) -> u32 {
        self.data.len() as u32
    }

    fn get_memory_size(&self) -> u32 {
        self.data.iter().map(|value| value.deep_size_of() as u32).sum()
    }
}


pub type KlineCacheEntry = GenericCacheEntry<KlineKey>;

impl KlineCacheEntry {
    pub fn new(key: KlineKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: get_utc8_timestamp_millis(),
            update_time: get_utc8_timestamp_millis(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}

impl From<KlineCacheEntry> for CacheEntry {
    fn from(entry: KlineCacheEntry) -> Self {
        CacheEntry::HistoryKline(entry)
    }
}

pub type IndicatorCacheEntry = GenericCacheEntry<IndicatorKey>;


impl From<IndicatorCacheEntry> for CacheEntry {
    fn from(entry: IndicatorCacheEntry) -> Self {
        CacheEntry::HistoryIndicator(entry)
    }
}

impl IndicatorCacheEntry {
    pub fn new(key: IndicatorKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: get_utc8_timestamp_millis(),
            update_time: get_utc8_timestamp_millis(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}


