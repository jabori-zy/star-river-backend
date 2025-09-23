use super::{
    BacktestStrategyContext, BacktestStrategyError, CacheEngineResponse, CacheValue, EngineResponse,
    EventCenterSingleton, GetCacheLengthMultiParams, GetCacheParams, GetDataFailedSnafu, Key, PlayIndex, StatsSnapshot,
    VirtualOrder, VirtualPosition, VirtualTransaction,
};
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::instrument;

impl BacktestStrategyContext {
    // 获取所有的virtual order
    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let virtual_orders = virtual_trading_system.get_virtual_orders();
        virtual_orders
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let current_positions = virtual_trading_system.get_current_positions_ref();
        current_positions.clone()
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let history_positions = virtual_trading_system.get_history_positions();
        history_positions
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let transactions = virtual_trading_system.get_transactions();
        transactions
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        let strategy_stats = self.strategy_stats.read().await;
        strategy_stats.get_stats_history(play_index).await
    }

    // 初始化信号计数
    #[instrument(skip(self))]
    pub async fn get_signal_count(&mut self) -> Result<i32, String> {
        // // 初始化信号计数
        // let min_cache_length = self.cache_lengths.values().min().cloned().unwrap_or(0);
        // Ok(min_cache_length as i32)
        let min_interval_keys: Vec<Key> = self
            .get_min_interval_symbols()
            .iter()
            .map(|key| Key::from(key.clone()))
            .collect();

        // 1. 从缓存引擎获取每一个symbol的缓存长度
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_length_params = GetCacheLengthMultiParams::new(
            self.strategy_id,
            min_interval_keys.clone(),
            self.strategy_name.clone(),
            resp_tx,
        );
        EventCenterSingleton::send_command(get_cache_length_params.into())
            .await
            .unwrap();
        let response = resp_rx.await.unwrap();
        if response.success() {
            let cache_engine_response = CacheEngineResponse::try_from(response);
            if let Ok(cache_engine_response) = cache_engine_response {
                match cache_engine_response {
                    CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response) => {
                        let symbol_cache_lengths = get_cache_length_multi_response.cache_length;

                        // 2. 判断每一个symbol的缓存长度是否相同
                        if symbol_cache_lengths.is_empty() {
                            return Err("no symbol cache lengths found".to_string());
                        }

                        // 获取第一个symbol的缓存长度作为基准
                        let min_cache_length = symbol_cache_lengths.values().min().cloned().unwrap_or(0);

                        // 检查所有symbol的缓存长度是否都相同
                        for (key, cache_length) in symbol_cache_lengths.iter() {
                            if *cache_length != min_cache_length {
                                return Err(format!(
                                    "symbol {} cache length {} is not same as min cache length {}",
                                    key.get_symbol(),
                                    cache_length,
                                    min_cache_length
                                ));
                            }
                        }

                        return Ok(min_cache_length as i32);
                    }
                    _ => {
                        return Err("get cache length multi failed".to_string());
                    }
                }
            } else {
                return Err("try from response failed".to_string());
            }
        } else {
            return Err("get cache length multi failed".to_string());
        }
    }

    pub async fn get_strategy_data(
        &self,
        play_index: PlayIndex,
        key: Key,
    ) -> Result<Vec<Arc<CacheValue>>, BacktestStrategyError> {
        // 安全检查：验证key是否属于当前策略
        let keys_map = self.keys.read().await;
        if !keys_map.contains_key(&key) {
            return Err(GetDataFailedSnafu {
                strategy_name: self.strategy_name.clone(),
                key: key.get_key_str(),
                play_index: play_index as u32,
            }
            .fail()?);
        }
        drop(keys_map); // 释放锁

        let (resp_tx, resp_rx) = oneshot::channel();

        let index = match &key {
            Key::Kline(kline_key) => {
                if self.min_interval_symbols.contains(kline_key) {
                    Some(play_index as u32)
                } else {
                    None
                }
            }
            Key::Indicator(indicator_key) => {
                let kline_key = indicator_key.get_kline_key();
                if self.min_interval_symbols.contains(&kline_key) {
                    Some(play_index as u32)
                } else {
                    None
                }
            }
        };

        let get_cache_params = GetCacheParams::new(
            self.strategy_id,
            "".to_string(),
            key.clone(),
            index,
            None,
            "".to_string(),
            resp_tx,
        );

        EventCenterSingleton::send_command(get_cache_params.into())
            .await
            .unwrap();

        let response = resp_rx.await.unwrap();

        if response.success() {
            match response {
                EngineResponse::CacheEngine(CacheEngineResponse::GetCacheData(get_cache_data_response)) => {
                    Ok(get_cache_data_response.cache_data)
                }
                _ => Err(GetDataFailedSnafu {
                    strategy_name: self.strategy_name.clone(),
                    key: key.get_key_str(),
                    play_index: play_index as u32,
                }
                .fail()?),
            }
        } else {
            Err(GetDataFailedSnafu {
                strategy_name: self.strategy_name.clone(),
                key: key.get_key_str(),
                play_index: play_index as u32,
            }
            .fail()?)
        }
    }
}
