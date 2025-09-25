use super::{
    BacktestStrategyContext, BacktestStrategyError, CacheValue,
    EventCenterSingleton, GetDataFailedSnafu, Key, PlayIndex, StatsSnapshot,
    VirtualOrder, VirtualPosition, VirtualTransaction, KlineDataLengthNotSameSnafu,
};
use std::sync::Arc;
use event_center::communication::{engine::cache_engine::{CacheEngineCommand, GetCacheCmdPayload, GetCacheCommand, GetCacheLengthMultiCmdPayload, GetCacheLengthMultiCommand}, Response};
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


    pub async fn get_signal_count_new(&self) -> Result<i32, BacktestStrategyError> {
        let kline_data_guard = self.kline_data.read().await;
        // 获取每一个key对应列表的长度
        let kline_data_lengths = kline_data_guard.values().map(|data| data.len() as i32).collect::<Vec<i32>>();
        
        // 如果只有一个key，则返回该key的长度
        if kline_data_lengths.len() == 1 {
            return Ok(kline_data_lengths[0]);
        } else {
            // 如果有多个key，比较列表中的长度是否全部相同
            if kline_data_lengths.iter().all(|length| length == &kline_data_lengths[0]) {
                return Ok(kline_data_lengths[0]);
            } else {
                return Err(KlineDataLengthNotSameSnafu {
                    strategy_name: self.strategy_name.clone(),
                }
                .build());
            }
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

        let payload = GetCacheCmdPayload::new(
            self.strategy_id,
            "".to_string(),
            key.clone(),
            index,
            None,
        );
        let cmd: CacheEngineCommand = GetCacheCommand::new(
            self.strategy_name.clone(),
            resp_tx,
            Some(payload),
        ).into();
        

        EventCenterSingleton::send_command(cmd.into())
            .await
            .unwrap();

        let response = resp_rx.await.unwrap();

        if response.is_success() {
             Ok(response.data.clone())
        } else {
            Err(GetDataFailedSnafu {
                strategy_name: self.strategy_name.clone(),
                key: key.get_key_str(),
                play_index: play_index as u32,
            }
            .fail()?)
        }
    }


    pub(super) async fn clear_data(&mut self) {
        let mut kline_data_guard = self.kline_data.write().await;
        kline_data_guard.iter_mut().for_each(|(key, kline_data)| {
            if !self.min_interval_symbols.contains(key) {
                kline_data.clear();
            }
        });
        let mut indicator_data_guard = self.indicator_data.write().await;
        indicator_data_guard.iter_mut().for_each(|(key, indicator_data)| {
            let kline_key = key.get_kline_key();
            if !self.min_interval_symbols.contains(&kline_key) {
                indicator_data.clear();
            }
        });
    }
}
