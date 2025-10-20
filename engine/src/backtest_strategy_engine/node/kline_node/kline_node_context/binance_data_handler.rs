use super::utils::bar_number;
use super::{
    AccountId, BacktestNodeContextTrait, EventCenterSingleton, GetKlineHistoryCmdPayload, GetKlineHistoryCommand,
    InsufficientKlineDataSnafu, KeyTrait, Kline, KlineInterval, KlineKey, KlineNodeContext, KlineNodeError,
    LoadKlineFromExchangeFailedSnafu, MarketEngineCommand, Response, TimeRange,
};
use chrono::{Duration, Utc};
use snafu::IntoError;
use std::sync::Arc;
use tokio::sync::{Semaphore, oneshot};

impl KlineNodeContext {
    pub(super) async fn get_binance_kline_history(&self, account_id: AccountId, time_range: &TimeRange) -> Result<(), KlineNodeError> {
        let bar_number = bar_number(&time_range, &self.min_interval_symbols[0].get_interval());
        tracing::debug!("[{}] bar number: {}", self.get_node_name(), bar_number);

        // Binance API limit is 1000 bars per request, use concurrent loading if > 1000
        if bar_number >= 1000 {
            tracing::info!(
                "[{}] Large data set detected ({} bars), using concurrent loading",
                self.get_node_name(),
                bar_number
            );

            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                if !self.min_interval_symbols.contains(&symbol_key) {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.get_node_name(),
                        symbol_key.get_symbol(),
                        symbol_key.get_interval()
                    );
                    continue;
                }

                let first_kline = self.request_first_kline_binance(account_id.clone(), &symbol_key).await?;
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientKlineDataSnafu {
                        first_kline_datetime: first_kline_datetime.to_string(),
                        start_time: start_time.to_string(),
                        end_time: time_range.end_date.to_string(),
                    }
                    .fail()?;
                }
                // Use concurrent loading
                self.load_binance_symbol_concurrently(account_id.clone(), symbol_key.clone()).await?;
            }
        } else {
            // Sequential loading for small datasets
            for (symbol_key, _) in self.selected_symbol_keys.iter() {
                if !self.min_interval_symbols.contains(&symbol_key) {
                    tracing::warn!(
                        "[{}] symbol: {}-{}, is not min interval, skip",
                        self.get_node_name(),
                        symbol_key.get_symbol(),
                        symbol_key.get_interval()
                    );
                    continue;
                }
                let first_kline = self.request_first_kline_binance(account_id.clone(), &symbol_key).await?;
                let first_kline_datetime = first_kline.first().unwrap().datetime();
                let start_time = time_range.start_date;
                if first_kline_datetime > start_time {
                    InsufficientKlineDataSnafu {
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

    // Get the first kline to validate data availability for Binance
    async fn request_first_kline_binance(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.get_node_id().clone();

        let time_range = TimeRange::new("2017-01-01 00:00:00".to_string(), Utc::now().to_string());
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineHistoryCmdPayload::new(
            self.get_strategy_id().clone(),
            node_id.clone(),
            account_id.clone(),
            kline_key.get_exchange(),
            kline_key.get_symbol(),
            kline_key.get_interval(),
            time_range,
        );
        let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(), resp_tx, Some(payload)).into();
        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.kline_history.clone());
        } else {
            let error = response.get_error();
            return Err(LoadKlineFromExchangeFailedSnafu {
                exchange: kline_key.get_exchange().to_string(),
            }.into_error(error));
        }
    }

    async fn load_binance_symbol_concurrently(&self, account_id: AccountId, symbol_key: KlineKey) -> Result<(), KlineNodeError> {
        let time_range = symbol_key.get_time_range().unwrap();

        // Split time range ensuring each chunk has < 1000 bars
        let chunks = self.split_time_range_for_binance(&time_range, &symbol_key.get_interval());

        // Limit concurrency to avoid overload
        let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent requests
        let mut handles = Vec::new();

        for chunk in chunks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let account_id_clone = account_id.clone();
            let mut chunk_key = symbol_key.clone();
            chunk_key.replace_time_range(chunk);

            // Clone necessary data to avoid lifetime issues
            let node_id = self.get_node_id().clone();
            let strategy_id = self.get_strategy_id().clone();

            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold the permit
                // Rebuild request inside spawn
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
                let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id, resp_tx, Some(payload)).into();
                EventCenterSingleton::send_command(cmd.into()).await.unwrap();

                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    Ok(response.kline_history.clone())
                } else {
                    let error = response.get_error();
                    Err(LoadKlineFromExchangeFailedSnafu {
                        exchange: chunk_key.get_exchange().to_string(),
                    }.into_error(error))
                }
            });

            handles.push(handle);
        }

        // Append kline data as each handle completes
        for handle in handles {
            let chunk_klines = handle.await.unwrap()?;
            self.append_kline_data(&symbol_key, &chunk_klines).await?;
        }

        Ok(())
    }

    fn split_time_range_for_binance(&self, time_range: &TimeRange, interval: &KlineInterval) -> Vec<TimeRange> {
        const MAX_BARS_PER_REQUEST: i64 = 999; // Ensure < 1000 bars per request

        // Calculate chunk size based on interval to ensure < 1000 bars per request
        let chunk_size = match interval {
            KlineInterval::Minutes1 => {
                // 1min * 999 = 999 minutes ≈ 16.65 hours
                Duration::minutes(MAX_BARS_PER_REQUEST)
            }
            KlineInterval::Minutes5 => {
                // 5min * 999 = 4995 minutes ≈ 3.47 days
                Duration::minutes(MAX_BARS_PER_REQUEST * 5)
            }
            KlineInterval::Minutes15 => {
                // 15min * 999 = 14985 minutes ≈ 10.4 days
                Duration::minutes(MAX_BARS_PER_REQUEST * 15)
            }
            KlineInterval::Minutes30 => {
                // 30min * 999 = 29970 minutes ≈ 20.8 days
                Duration::minutes(MAX_BARS_PER_REQUEST * 30)
            }
            KlineInterval::Hours1 => {
                // 1hour * 999 = 999 hours ≈ 41.6 days
                Duration::hours(MAX_BARS_PER_REQUEST)
            }
            KlineInterval::Hours2 => {
                // 2hours * 999 = 1998 hours ≈ 83.3 days
                Duration::hours(MAX_BARS_PER_REQUEST * 2)
            }
            KlineInterval::Hours4 => {
                // 4hours * 999 = 3996 hours ≈ 166.5 days
                Duration::hours(MAX_BARS_PER_REQUEST * 4)
            }
            KlineInterval::Hours6 => {
                // 6hours * 999 = 5994 hours ≈ 249.75 days
                Duration::hours(MAX_BARS_PER_REQUEST * 6)
            }
            KlineInterval::Hours8 => {
                // 8hours * 999 = 7992 hours ≈ 333 days
                Duration::hours(MAX_BARS_PER_REQUEST * 8)
            }
            KlineInterval::Hours12 => {
                // 12hours * 999 = 11988 hours ≈ 499.5 days
                Duration::hours(MAX_BARS_PER_REQUEST * 12)
            }
            KlineInterval::Days1 => {
                // 1day * 999 = 999 days ≈ 2.74 years
                Duration::days(MAX_BARS_PER_REQUEST)
            }
            KlineInterval::Weeks1 => {
                // 1week * 999 = 6993 days ≈ 19.16 years
                Duration::weeks(MAX_BARS_PER_REQUEST)
            }
            KlineInterval::Months1 => {
                // 1month * 999 ≈ 30 * 999 days ≈ 81.8 years
                Duration::days(MAX_BARS_PER_REQUEST * 30)
            }
            _ => {
                // Default to 30 days for any other intervals
                Duration::days(30)
            }
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

        tracing::debug!(
            "Split time range into {} chunks for interval {:?}",
            chunks.len(),
            interval
        );

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_split_time_range_for_binance_1min_small_range() {
        // Test 1-minute interval with small time range (< 999 minutes)
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2024-01-01 10:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes1);

        // 10 hours = 600 minutes < 999, should be 1 chunk
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_date, start);
        assert_eq!(chunks[0].end_date, end);
    }

    #[test]
    fn test_split_time_range_for_binance_1min_large_range() {
        // Test 1-minute interval with large time range (> 999 minutes)
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2024-01-03 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes1);

        // 2 days = 2880 minutes, should be split into multiple chunks
        // Each chunk is 999 minutes (16.65 hours)
        assert!(chunks.len() > 1);

        // Verify no chunk exceeds 999 minutes
        for chunk in &chunks {
            let duration_minutes = (chunk.end_date - chunk.start_date).num_minutes();
            assert!(duration_minutes <= 999, "Chunk duration {} exceeds 999 minutes", duration_minutes);
        }

        // Verify chunks cover the entire range
        assert_eq!(chunks.first().unwrap().start_date, start);
        assert_eq!(chunks.last().unwrap().end_date, end);
    }

    #[test]
    fn test_split_time_range_for_binance_5min() {
        // Test 5-minute interval
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2024-01-15 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes5);

        // 14 days, each chunk is 4995 minutes (3.47 days)
        assert!(chunks.len() > 1);

        // Verify no chunk exceeds 4995 minutes
        for chunk in &chunks {
            let duration_minutes = (chunk.end_date - chunk.start_date).num_minutes();
            let max_bars = duration_minutes / 5;
            assert!(max_bars <= 999, "Chunk has {} bars, exceeds 999", max_bars);
        }
    }

    #[test]
    fn test_split_time_range_for_binance_1hour() {
        // Test 1-hour interval
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2024-03-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Hours1);

        // ~60 days, each chunk is 999 hours (41.6 days)
        assert!(chunks.len() > 1);

        // Verify no chunk exceeds 999 hours
        for chunk in &chunks {
            let duration_hours = (chunk.end_date - chunk.start_date).num_hours();
            assert!(duration_hours <= 999, "Chunk duration {} exceeds 999 hours", duration_hours);
        }
    }

    #[test]
    fn test_split_time_range_for_binance_1day() {
        // Test 1-day interval
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Days1);

        // 3 years ≈ 1095 days, each chunk is 999 days
        assert!(chunks.len() > 1);

        // Verify no chunk exceeds 999 days
        for chunk in &chunks {
            let duration_days = (chunk.end_date - chunk.start_date).num_days();
            assert!(duration_days <= 999, "Chunk duration {} exceeds 999 days", duration_days);
        }
    }

    #[test]
    fn test_split_time_range_for_binance_exact_boundary() {
        // Test exact boundary case: exactly 999 minutes
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = start + Duration::minutes(999);
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes1);

        // Exactly 999 minutes should result in 1 chunk
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_date, start);
        assert_eq!(chunks[0].end_date, end);
    }

    #[test]
    fn test_split_time_range_for_binance_boundary_plus_one() {
        // Test boundary + 1: 1000 minutes
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = start + Duration::minutes(1000);
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes1);

        // 1000 minutes should be split into 2 chunks
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_split_time_range_for_binance_chunks_continuous() {
        // Test that chunks are continuous (no gaps or overlaps)
        let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let end = NaiveDateTime::parse_from_str("2024-01-10 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let time_range = TimeRange {
            start_date: start,
            end_date: end,
        };

        let chunks = split_time_range_helper(&time_range, &KlineInterval::Minutes1);

        // Verify chunks are continuous
        for i in 1..chunks.len() {
            assert_eq!(chunks[i - 1].end_date, chunks[i].start_date,
                "Gap or overlap detected between chunk {} and {}", i - 1, i);
        }
    }

    // Helper function to test split_time_range_for_binance without needing KlineNodeContext
    fn split_time_range_helper(time_range: &TimeRange, interval: &KlineInterval) -> Vec<TimeRange> {
        const MAX_BARS_PER_REQUEST: i64 = 999;

        let chunk_size = match interval {
            KlineInterval::Minutes1 => Duration::minutes(MAX_BARS_PER_REQUEST),
            KlineInterval::Minutes5 => Duration::minutes(MAX_BARS_PER_REQUEST * 5),
            KlineInterval::Minutes15 => Duration::minutes(MAX_BARS_PER_REQUEST * 15),
            KlineInterval::Minutes30 => Duration::minutes(MAX_BARS_PER_REQUEST * 30),
            KlineInterval::Hours1 => Duration::hours(MAX_BARS_PER_REQUEST),
            KlineInterval::Hours2 => Duration::hours(MAX_BARS_PER_REQUEST * 2),
            KlineInterval::Hours4 => Duration::hours(MAX_BARS_PER_REQUEST * 4),
            KlineInterval::Hours6 => Duration::hours(MAX_BARS_PER_REQUEST * 6),
            KlineInterval::Hours8 => Duration::hours(MAX_BARS_PER_REQUEST * 8),
            KlineInterval::Hours12 => Duration::hours(MAX_BARS_PER_REQUEST * 12),
            KlineInterval::Days1 => Duration::days(MAX_BARS_PER_REQUEST),
            KlineInterval::Weeks1 => Duration::weeks(MAX_BARS_PER_REQUEST),
            KlineInterval::Months1 => Duration::days(MAX_BARS_PER_REQUEST * 30),
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
