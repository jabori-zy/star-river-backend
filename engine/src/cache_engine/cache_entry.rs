use chrono::Utc;
use sea_orm::prelude::DateTimeUtc;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::key::*;
use star_river_core::indicator::Indicator;
use star_river_core::market::{Kline, QuantData};
use deepsize::DeepSizeOf;
use std::collections::VecDeque;
use std::time::Duration;
use std::fmt::Debug;

pub trait CacheEntryTrait {

    type Key: KeyTrait;
    type Value: QuantData;

    fn get_key(&self) -> Self::Key;
    fn initialize(&mut self, value: Vec<Self::Value>);
    fn update(&mut self, value: Self::Value);
    fn clear(&mut self);
    fn get_all_data(&self) -> Vec<Self::Value>;
    fn get_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Self::Value>;
    fn get_create_time(&self) -> DateTimeUtc;
    fn get_update_time(&self) -> DateTimeUtc;
    fn get_max_size(&self) -> u32;
    fn get_is_fresh(&self) -> bool;
    fn get_ttl(&self) -> Duration;
    fn get_length(&self) -> u32;
    fn get_memory_size(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct GenericCacheEntry<K: KeyTrait, V: QuantData> {
    pub key: K,
    pub data: VecDeque<V>,
    pub create_time: DateTimeUtc,
    pub update_time: DateTimeUtc,
    pub max_size: Option<u32>, // 如果为None，则表示无限缓存
    pub is_fresh: bool,
    pub ttl: Duration,
}

impl<K: KeyTrait, V: QuantData + DeepSizeOf> GenericCacheEntry<K, V> {
    fn add_new_data(&mut self, value: V) {
        if self.max_size.is_none() {
            // 无限缓存
            self.data.push_back(value);
        } else {
            // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
            if self.data.len() >= self.max_size.unwrap() as usize {
                self.data.pop_front();
            }
            self.data.push_back(value);
        }
    }
}

impl<K: KeyTrait, V: QuantData + DeepSizeOf> CacheEntryTrait for GenericCacheEntry<K, V> {
    type Key = K;
    type Value = V;


    fn initialize(&mut self, data: Vec<Self::Value>) {
        self.data = data.into_iter().collect();
        self.is_fresh = true;
        self.update_time = Utc::now();
    }

    fn update(&mut self, value: Self::Value) {
        if let Some(last_data) = self.data.back() {
            // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
            if last_data.get_datetime() == value.get_datetime() {
                self.data.pop_back();
                self.data.push_back(value);
            } else {
                // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                self.add_new_data(value);
            }
        } else {
            // 如果没有数据，直接插入
            self.data.push_back(value);
        }
        self.update_time = Utc::now();
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn get_key(&self) -> Self::Key {
        self.key.clone()
    }

    fn get_all_data(&self) -> Vec<Self::Value> {
        self.data.iter().cloned().collect()
    }

    fn get_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Self::Value> {
        match (index, limit) {
            // 有index，有limit
            (Some(idx), Some(limit_val)) => {
                // 如果索引超出范围，返回空
                if idx as usize >= self.data.len() {
                    return Vec::new();
                }

                // 计算从索引开始向前取limit个元素
                let end = idx as usize + 1;
                let start = if limit_val as usize >= end {
                    0
                } else {
                    end - limit_val as usize
                };

                self.data.range(start..end).cloned().collect()
            }

            // 有index，无limit
            (Some(idx), None) => {
                // 如果索引超出范围，返回空
                if idx as usize >= self.data.len() {
                    return Vec::new();
                }

                // 从索引开始向前取所有元素（到开头）
                let end = idx as usize + 1;
                self.data.range(0..end).cloned().collect()
            }

            // 无index，有limit
            (None, Some(limit_val)) => {
                // 从后往前取limit条数据
                if limit_val as usize >= self.data.len() {
                    return self.get_all_data();
                }

                let start = self.data.len().saturating_sub(limit_val as usize);
                self.data.range(start..).cloned().collect()
            }

            // 无index，无limit
            (None, None) => {
                // 如果limit和index都为None，则返回所有数据
                self.get_all_data()
            }
        }
    }

    fn get_create_time(&self) -> DateTimeUtc {
        self.create_time
    }

    fn get_update_time(&self) -> DateTimeUtc {
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

pub type KlineCacheEntry = GenericCacheEntry<KlineKey, Kline>;

impl KlineCacheEntry {
    pub fn new(key: KlineKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: Utc::now(),
            update_time: Utc::now(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}


pub type IndicatorCacheEntry = GenericCacheEntry<IndicatorKey, Indicator>;


impl IndicatorCacheEntry {
    pub fn new(key: IndicatorKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: Utc::now(),
            update_time: Utc::now(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}
