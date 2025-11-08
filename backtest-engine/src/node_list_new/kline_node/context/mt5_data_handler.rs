// std
use std::sync::Arc;

// third-party
use chrono::Duration;
use snafu::IntoError;
use tokio::sync::{Semaphore, oneshot};

// workspace crate

use star_river_core::custom_type::AccountId;
use strategy_core::strategy::TimeRange;
use key::{KeyTrait, KlineKey};
use strategy_core::node::context_trait::NodeIdentityExt;
use star_river_event::communication::{GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand};
use event_center_new::EventCenterSingleton;
use star_river_core::kline::{Kline, KlineInterval};
use event_center_core::communication::response::Response;

// current crate
use super::{
    KlineNodeContext, KlineNodeError,
    utils::bar_number,
};
use crate::{
    error::node_error::kline_node_error::{InsufficientMetaTrader5KlineDataSnafu, LoadKlineFromExchangeFailedSnafu},
};

impl KlineNodeContext {
    pub(super) async fn get_mt5_kline_history(&self, account_id: AccountId, time_range: &TimeRange) -> Result<(), KlineNodeError> {
        let bar_number = bar_number(&time_range, &self.min_interval_symbols[0].get_interval());
        tracing::debug!("[{}] bar number: {}", self.node_name(), bar_number);

        // 如果大于2000条, 则开启多线程加载
        if bar_number >= 10000 {
            tracing::info!(
                "[{}] Large data set detected ({} bars), using concurrent loading",
                self.node_name(),
                bar_number
            );

            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                if !self.min_interval_symbols.contains(&symbol_key) {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.node_name(),
                        symbol_key.get_symbol(),
                        symbol_key.get_interval()
                    );
                    continue;
                }

                let first_kline = self.request_first_kline_from_mt5(account_id.clone(), &symbol_key).await?;
                // 第一根k线的时间
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                // 如果第一根k线的时间小于start_time，则报错
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientMetaTrader5KlineDataSnafu {
                        first_kline_datetime: first_kline_datetime.to_string(),
                        start_time: start_time.to_string(),
                        end_time: time_range.end_date.to_string(),
                    }
                    .fail()?;
                }
                // 使用并发加载
                self.load_symbol_concurrently_from_mt5(account_id.clone(), symbol_key.clone())
                    .await?;
            }
        } else {
            // 遍历每一个symbol，从交易所获取k线历史
            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                // 如果key不在最小周期交易对列表中，则跳过
                if !self.min_interval_symbols.contains(&symbol_key) {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.node_name(),
                        symbol_key.get_symbol(),
                        symbol_key.get_interval()
                    );
                    continue;
                }
                let first_kline = self.request_first_kline_from_mt5(account_id.clone(), &symbol_key).await?;
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientMetaTrader5KlineDataSnafu {
                        first_kline_datetime: first_kline_datetime.to_string(),
                        start_time: start_time.to_string(),
                        end_time: time_range.end_date.to_string(),
                    }
                    .fail()?;
                }

                let kline_history = self.request_kline_history(account_id.clone(), symbol_key).await?;
                self.init_strategy_kline_data(symbol_key, &kline_history).await?;
            }
        }

        Ok(())
    }

    // 获取第一个k线
    async fn request_first_kline_from_mt5(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.node_id().clone();

        let time_range = TimeRange::new("1971-01-01 00:00:00".to_string(), "1971-01-02 00:00:00".to_string());
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineHistoryCmdPayload::new(
            self.strategy_id().clone(),
            node_id.clone(),
            account_id.clone(),
            kline_key.get_exchange(),
            kline_key.get_symbol(),
            kline_key.get_interval(),
            time_range,
        );
        let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.kline_history.clone());
            }
            Response::Fail { error, .. } => {
                return Err(LoadKlineFromExchangeFailedSnafu {
                    exchange: kline_key.get_exchange().to_string(),
                }
                .into_error(error));
            }
        }
    }

    async fn load_symbol_concurrently_from_mt5(&self, account_id: AccountId, symbol_key: KlineKey) -> Result<(), KlineNodeError> {
        let time_range = symbol_key.get_time_range().unwrap();

        // 根据时间范围大小决定分片策略
        let chunks = self.split_time_range_for_mt5(&time_range, &symbol_key.get_interval());

        // 限制并发数量，避免过载
        let semaphore = Arc::new(Semaphore::new(5)); // 最多5个并发请求
        let mut handles = Vec::new();

        for chunk in chunks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let account_id_clone = account_id.clone();
            let mut chunk_key = symbol_key.clone();
            chunk_key.replace_time_range(chunk);

            // 克隆必要的数据以避免生命周期问题
            let node_id = self.node_id().clone();
            let strategy_id = self.strategy_id().clone();

            let handle = tokio::spawn(async move {
                let _permit = permit; // 持有许可证
                // 在spawn内部重新构建请求
                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = GetKlineHistoryCmdPayload::new(
                    strategy_id,
                    node_id.clone(),
                    account_id_clone,
                    chunk_key.get_exchange(),
                    chunk_key.get_symbol(),
                    chunk_key.get_interval(),
                    chunk_key.get_time_range().unwrap(),
                );
                let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id, resp_tx, payload).into();
                EventCenterSingleton::send_command(cmd.into()).await.unwrap();

                let response = resp_rx.await.unwrap();
                match response {
                    Response::Success { payload, .. } => {
                        Ok(payload.kline_history.clone())
                    }
                    Response::Fail { error, .. } => {
                        return Err(LoadKlineFromExchangeFailedSnafu {
                            exchange: chunk_key.get_exchange().to_string(),
                        }
                        .into_error(error));
                    }
                }
            });

            handles.push(handle);
        }

        // 每个handle完成时，发送append kline data命令
        for handle in handles {
            let chunk_klines = handle.await.unwrap()?;
            self.append_kline_data(&symbol_key, &chunk_klines).await?;
        }

        Ok(())
    }

    fn split_time_range_for_mt5(&self, time_range: &TimeRange, interval: &KlineInterval) -> Vec<TimeRange> {
        let total_duration = time_range.duration();

        // 根据K线周期和总时长计算合适的分片大小
        let chunk_size = match interval {
            KlineInterval::Minutes1 => {
                if total_duration.num_days() > 7 {
                    Duration::days(1) // 1分钟K线，每次请求1天
                } else {
                    total_duration // 小于7天直接请求
                }
            }
            KlineInterval::Minutes5 => Duration::days(3),
            KlineInterval::Minutes15 => Duration::days(7),
            KlineInterval::Hours1 => Duration::days(30),
            KlineInterval::Days1 => Duration::days(365),
            _ => Duration::days(30),
        };

        let mut chunks = Vec::new();
        let mut current_start = time_range.start_date;

        while current_start < time_range.end_date {
            let chunk_end = std::cmp::min(current_start + chunk_size, time_range.end_date);

            chunks.push(TimeRange {
                start_date: current_start,
                end_date: chunk_end,
            });

            current_start = chunk_end;
        }

        chunks
    }
}
