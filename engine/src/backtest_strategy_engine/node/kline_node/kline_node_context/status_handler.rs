use super::{
    BacktestNodeContextTrait, EngineResponse, EventCenterSingleton, ExchangeEngineCommand, GetKlineHistoryCmdPayload,
    GetKlineHistoryCommand, InitKlineDataCmdPayload, InitKlineDataCommand, KeyTrait, KlineNodeContext, MarketEngineCommand,
    RegisterExchangeCmdPayload, RegisterExchangeCommand, RegisterExchangeRespPayload, Response, Kline, KlineNodeError, KlineKey,
    AccountId, LoadKlineHistoryFromExchangeFailedSnafu, InitKlineDataFailedSnafu, TimeRange, KlineInterval, AppendKlineDataCmdPayload, 
    AppendKlineDataCommand, AppendKlineDataFailedSnafu, InsufficientKlineDataSnafu, Exchange
};
use tokio::sync::{oneshot, Semaphore};
use tracing::instrument;
use snafu::IntoError;
use super::utils::bar_number;
use chrono::Duration;
use std::sync::Arc;

impl KlineNodeContext {
    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<(), KlineNodeError> {
        tracing::info!("[{}] start to load backtest kline data from exchange", self.get_node_name());

        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();

        let exchange = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();

        let time_range = self.backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        match exchange {
            Exchange::Metatrader5(_) => {
                // 计算理论上的k线数量
                let bar_number = bar_number(&time_range, &self.min_interval_symbols[0].get_interval());
                tracing::debug!("[{}] bar number: {}", self.get_node_name(), bar_number);
                
                // 如果大于2000条, 则开启多线程加载
                if bar_number >= 10000 {
                    tracing::info!("[{}] Large data set detected ({} bars), using concurrent loading", self.get_node_name(), bar_number);

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

                        let first_kline = self.request_first_kline(account_id.clone(), &symbol_key).await?;
                        // 第一根k线的时间
                        let first_kline_datetime = first_kline.first().unwrap().datetime();
                        // 如果第一根k线的时间小于start_time，则报错
                        let start_time = time_range.start_date;
                        if first_kline_datetime > start_time {
                            InsufficientKlineDataSnafu{
                                first_kline_datetime: first_kline_datetime.to_string(),
                                start_time: start_time.to_string(),
                                end_time: time_range.end_date.to_string(),
                            }
                            .fail()?;

                        }
                        // 使用并发加载
                        self.load_symbol_concurrently(account_id.clone(), symbol_key.clone()).await?;
                    }
                } else {
                    // 遍历每一个symbol，从交易所获取k线历史
                    for (symbol_key, _) in self.selected_symbol_keys.iter() {
                        // 如果key不在最小周期交易对列表中，则跳过
                        if !self.min_interval_symbols.contains(&symbol_key) {
                            tracing::warn!(
                                "[{}] symbol: {}-{}, is not min interval, skip",
                                self.get_node_name(),
                                symbol_key.get_symbol(),
                                symbol_key.get_interval()
                            );
                            continue;
                        }
                        let first_kline = self.request_first_kline(account_id.clone(), &symbol_key).await?;
                        let first_kline_datetime = first_kline.first().unwrap().datetime();
                        let start_time = time_range.start_date;
                        if first_kline_datetime > start_time {
                            InsufficientKlineDataSnafu{
                                first_kline_datetime: first_kline_datetime.to_string(),
                                start_time: start_time.to_string(),
                                end_time: time_range.end_date.to_string(),
                            }
                            .fail()?;

                        }

                        let kline_history = self.request_kline_history(account_id.clone(), symbol_key).await?;
                        self.init_kline_data(symbol_key, &kline_history).await?;
                    }

                }


            }

            Exchange::Binance => {
                // 遍历每一个symbol，从交易所获取k线历史
                for (symbol_key, _) in self.selected_symbol_keys.iter() {
                    // 如果key不在最小周期交易对列表中，则跳过
                    if !self.min_interval_symbols.contains(&symbol_key) {
                        tracing::warn!(
                            "[{}] symbol: {}-{}, is not min interval, skip",
                            self.get_node_name(),
                            symbol_key.get_symbol(),
                            symbol_key.get_interval()
                        );
                        continue;
                    }

                    let kline_history = self.request_kline_history(account_id.clone(), symbol_key).await?;
                    self.init_kline_data(symbol_key, &kline_history).await?;

                }

            }
            _ => {
                return Ok(());
            }
        }

        
        
        Ok(())
    }

    fn split_time_range(&self, time_range: &TimeRange, interval: &KlineInterval) -> Vec<TimeRange> {
        let total_duration = time_range.duration();

        // 根据K线周期和总时长计算合适的分片大小
        let chunk_size = match interval {
            KlineInterval::Minutes1 => {
                if total_duration.num_days() > 7 {
                    Duration::days(1) // 1分钟K线，每次请求1天
                } else {
                    total_duration // 小于7天直接请求
                }
            },
            KlineInterval::Minutes5 => Duration::days(3),
            KlineInterval::Minutes15 => Duration::days(7),
            KlineInterval::Hours1 => Duration::days(30),
            KlineInterval::Days1 => Duration::days(365),
            _ => Duration::days(30),
        };

        let mut chunks = Vec::new();
        let mut current_start = time_range.start_date;

        while current_start < time_range.end_date {
            let chunk_end = std::cmp::min(
                current_start + chunk_size,
                time_range.end_date
            );

            chunks.push(TimeRange {
                start_date: current_start,
                end_date: chunk_end,
            });

            current_start = chunk_end;
        }

        chunks
    }


    async fn load_symbol_concurrently(&self, account_id: AccountId, symbol_key: KlineKey) -> Result<(), KlineNodeError> {
        let time_range = symbol_key.get_time_range().unwrap();

        // 根据时间范围大小决定分片策略
        let chunks = self.split_time_range(&time_range, &symbol_key.get_interval());

        // 限制并发数量，避免过载
        let semaphore = Arc::new(Semaphore::new(5)); // 最多5个并发请求
        let mut handles = Vec::new();

        for chunk in chunks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let account_id_clone = account_id.clone();
            let mut chunk_key = symbol_key.clone();
            chunk_key.replace_time_range(chunk);

            // 克隆必要的数据以避免生命周期问题
            let node_id = self.get_node_id().clone();
            let strategy_id = self.get_strategy_id().clone();

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
                let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id, resp_tx, Some(payload)).into();
                EventCenterSingleton::send_command(cmd.into()).await.unwrap();

                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    Ok(response.kline_history.clone())
                } else {
                    let error = response.get_error();
                    Err(LoadKlineHistoryFromExchangeFailedSnafu{}.into_error(error))
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


    async fn request_kline_history(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.get_node_id().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineHistoryCmdPayload::new(
            self.get_strategy_id().clone(),
            node_id.clone(),
            account_id.clone(),
            kline_key.get_exchange(),
            kline_key.get_symbol(),
            kline_key.get_interval(),
            kline_key.get_time_range().unwrap(),
        );
        let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(), resp_tx, Some(payload)).into();
        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.kline_history.clone());
        } else {
            let error = response.get_error();
            return Err(LoadKlineHistoryFromExchangeFailedSnafu{}.into_error(error));
        }

    }


    // 获取第一个k线
    async fn request_first_kline(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.get_node_id().clone();
        
        let time_range = TimeRange::new("1971-01-01 00:00:00".to_string(), "1971-01-02 00:00:00".to_string());
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
            return Err(LoadKlineHistoryFromExchangeFailedSnafu{}.into_error(error));
        }

    }


    async fn init_kline_data(&self, symbol_key: &KlineKey, kline_history: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = InitKlineDataCmdPayload::new(symbol_key.clone(), kline_history.clone());
        let init_kline_data_command = InitKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));
        self.get_strategy_command_sender().send(init_kline_data_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(());
        } else {
            let error = response.get_error();
            return Err(InitKlineDataFailedSnafu{}.into_error(error));
        }
    }


    async fn append_kline_data(&self, symbol_key: &KlineKey, kline_series: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = AppendKlineDataCmdPayload::new(symbol_key.clone(), kline_series.clone());
        let append_kline_data_command = AppendKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));
        self.get_strategy_command_sender().send(append_kline_data_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(());
        }
        let error = response.get_error();
        return Err(AppendKlineDataFailedSnafu{}.into_error(error));
    }
        
        
    

    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<EngineResponse<RegisterExchangeRespPayload>, String> {
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();

        tracing::info!("[{}] start to register exchange [{}]", node_name, exchange);

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = RegisterExchangeCmdPayload::new(account_id, exchange);
        let cmd: ExchangeEngineCommand = RegisterExchangeCommand::new(node_id, resp_tx, Some(payload)).into();

        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }
}
