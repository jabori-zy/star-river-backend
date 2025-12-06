// std
use std::sync::Arc;

// third-party
use chrono::Duration;
// workspace crate
use event_center::EventCenterSingleton;
use event_center_core::communication::response::Response;
use key::{KeyTrait, KlineKey};
use snafu::IntoError;
use star_river_core::{
    custom_type::AccountId,
    kline::{Kline, KlineInterval},
    system::TimeRange,
};
use star_river_event::communication::{GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand};
use strategy_core::node::context_trait::NodeInfoExt;
use tokio::sync::{Semaphore, oneshot};

// current crate
use super::{KlineNodeContext, KlineNodeError, utils::bar_number};
use crate::node::node_error::kline_node_error::{InsufficientBacktestDataForMetaTrader5Snafu, LoadKlineFromExchangeFailedSnafu};

impl KlineNodeContext {
    pub(super) async fn get_mt5_kline_history(&self, account_id: AccountId, time_range: &TimeRange) -> Result<(), KlineNodeError> {
        let bar_number = bar_number(&time_range, &self.min_interval);
        tracing::debug!("[{}] bar number: {}", self.node_name(), bar_number);

        // If greater than 2000 bars, enable multi-threaded loading
        if bar_number >= 10000 {
            tracing::info!(
                "[{}] Large data set detected ({} bars), using concurrent loading",
                self.node_name(),
                bar_number
            );

            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                if symbol_key.interval() != self.min_interval {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.node_name(),
                        symbol_key.symbol(),
                        symbol_key.interval()
                    );
                    continue;
                }

                let first_kline = self.request_first_kline_from_mt5(account_id.clone(), &symbol_key).await?;
                // Time of the first kline
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                // Error if first kline time is less than start_time
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientBacktestDataForMetaTrader5Snafu {
                        first_kline_datetime: first_kline_datetime.to_string(),
                        start_time: start_time.to_string(),
                        end_time: time_range.end_date.to_string(),
                    }
                    .fail()?;
                }
                // Use concurrent loading
                self.load_symbol_concurrently_from_mt5(account_id.clone(), symbol_key.clone())
                    .await?;
            }
        } else {
            // Iterate over each symbol to get kline history from exchange
            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                // Skip if key is not in minimum interval trading pair list
                if symbol_key.interval() != self.min_interval {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.node_name(),
                        symbol_key.symbol(),
                        symbol_key.interval()
                    );
                    continue;
                }
                let first_kline = self.request_first_kline_from_mt5(account_id.clone(), &symbol_key).await?;
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientBacktestDataForMetaTrader5Snafu {
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

    // Get the first kline
    async fn request_first_kline_from_mt5(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.node_id().clone();

        let time_range = TimeRange::new("1971-01-01 00:00:00".to_string(), "1971-01-02 00:00:00".to_string());
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineHistoryCmdPayload::new(
            self.strategy_id().clone(),
            node_id.clone(),
            account_id.clone(),
            kline_key.exchange(),
            kline_key.symbol(),
            kline_key.interval(),
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
                    exchange: kline_key.exchange().to_string(),
                }
                .into_error(error));
            }
        }
    }

    async fn load_symbol_concurrently_from_mt5(&self, account_id: AccountId, symbol_key: KlineKey) -> Result<(), KlineNodeError> {
        let time_range = symbol_key.time_range().unwrap();

        // Determine chunking strategy based on time range size
        let chunks = self.split_time_range_for_mt5(&time_range, &symbol_key.interval());

        // Limit concurrency to avoid overload
        let semaphore = Arc::new(Semaphore::new(5)); // Maximum 5 concurrent requests
        let mut handles = Vec::new();

        for chunk in chunks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let account_id_clone = account_id.clone();
            let mut chunk_key = symbol_key.clone();
            chunk_key.replace_time_range(chunk);

            // Clone necessary data to avoid lifetime issues
            let node_id = self.node_id().clone();
            let strategy_id = self.strategy_id().clone();

            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold the permit
                // Rebuild request inside spawn
                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = GetKlineHistoryCmdPayload::new(
                    strategy_id,
                    node_id.clone(),
                    account_id_clone,
                    chunk_key.exchange(),
                    chunk_key.symbol(),
                    chunk_key.interval(),
                    chunk_key.time_range().unwrap(),
                );
                let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id, resp_tx, payload).into();
                EventCenterSingleton::send_command(cmd.into()).await.unwrap();

                let response = resp_rx.await.unwrap();
                match response {
                    Response::Success { payload, .. } => Ok(payload.kline_history.clone()),
                    Response::Fail { error, .. } => {
                        return Err(LoadKlineFromExchangeFailedSnafu {
                            exchange: chunk_key.exchange().to_string(),
                        }
                        .into_error(error));
                    }
                }
            });

            handles.push(handle);
        }

        // When each handle completes, send append kline data command
        for handle in handles {
            let chunk_klines = handle.await.unwrap()?;
            self.append_kline_data(&symbol_key, &chunk_klines).await?;
        }

        Ok(())
    }

    fn split_time_range_for_mt5(&self, time_range: &TimeRange, interval: &KlineInterval) -> Vec<TimeRange> {
        let total_duration = time_range.duration();

        // Calculate appropriate chunk size based on kline interval and total duration
        let chunk_size = match interval {
            KlineInterval::Minutes1 => {
                if total_duration.num_days() > 7 {
                    Duration::days(1) // 1-minute klines, request 1 day each time
                } else {
                    total_duration // Less than 7 days, request directly
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
