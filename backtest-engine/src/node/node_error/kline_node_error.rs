use std::sync::Arc;

use event_center::EventCenterError;
use key::error::KeyError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};
use strategy_core::error::{NodeError, NodeStateMachineError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    NodeStateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    EventCenterError { source: EventCenterError, backtrace: Backtrace },

    #[snafu(transparent)]
    KeyError { source: KeyError, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] Exchange register failed"))]
    RegisterExchangeFailed {
        node_name: NodeName,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("load kline history from exchange failed."))]
    LoadKlineFromExchangeFailed {
        exchange: String,
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "the backtest time range is from {start_time} to {end_time}, but the first kline datetime is {first_kline_datetime} Go to Metatrader5 Terminal's Tools -> Options -> Charts and then set Max bars in charts to Unlimited. Then restart Metatrader5. "
    ))]
    InsufficientBacktestDataForMetaTrader5 {
        first_kline_datetime: String,
        start_time: String,
        end_time: String,
        backtrace: Backtrace,
    },
    #[snafu(display("@[{node_name}] exchange mode is not configured"))]
    ExchangeModeNotConfigured { node_name: NodeName, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] symbols is not configured"))]
    SymbolsIsNotConfiguredSnafu { node_name: NodeName, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] get min interval symbols failed"))]
    GetMinIntervalFromStrategyFailed {
        node_name: NodeName,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] first kline is empty for {exchange}-{symbol}-{interval}"))]
    FirstKlineIsEmpty {
        node_name: NodeName,
        exchange: String,
        symbol: String,
        interval: String,
        backtrace: Backtrace,
    },

    #[snafu(display("acquire semaphore failed"))]
    AcquireSemaphoreFailed {
        source: tokio::sync::AcquireError,
        backtrace: Backtrace,
    },

    #[snafu(display("fetch kline data task failed"))]
    FetchKlineDataTaskFailed {
        node_name: NodeName,
        exchange: String,
        symbol: String,
        interval: String,
        source: tokio::task::JoinError,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "the backtest time range is from {start_time} to {end_time}, but the first kline datetime of {exchange}-{symbol}-{interval} is {first_kline_datetime}"
    ))]
    InsufficientBacktestData {
        first_kline_datetime: String,
        symbol: String,
        interval: String,
        exchange: String,
        start_time: String,
        end_time: String,
        backtrace: Backtrace,
    },

    #[snafu(display("pending update kline not exist for {symbol}-{interval}"))]
    PendingUpdateKlineNotExist {
        symbol: String,
        interval: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for KlineNodeError {
    fn get_prefix(&self) -> &'static str {
        "KLINE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            KlineNodeError::NodeError { .. } => 1001,                              // node error
            KlineNodeError::NodeStateMachineError { .. } => 1002,                  // node state machine error
            KlineNodeError::EventCenterError { .. } => 1003,                       // event center error
            KlineNodeError::KeyError { .. } => 1004,                               // key error
            KlineNodeError::RegisterExchangeFailed { .. } => 1005,                 // register exchange failed
            KlineNodeError::LoadKlineFromExchangeFailed { .. } => 1006,            // load kline from exchange failed
            KlineNodeError::InsufficientBacktestDataForMetaTrader5 { .. } => 1007, // insufficient meta trader 5 kline data
            KlineNodeError::ExchangeModeNotConfigured { .. } => 1008,              // exchange mode is not configured
            KlineNodeError::SymbolsIsNotConfiguredSnafu { .. } => 1009,            // symbols is not configured
            KlineNodeError::GetMinIntervalFromStrategyFailed { .. } => 1010,       // get min interval symbols from strategy failed
            KlineNodeError::FirstKlineIsEmpty { .. } => 1011,                      // first kline is empty
            KlineNodeError::AcquireSemaphoreFailed { .. } => 1012,                 // acquire semaphore failed
            KlineNodeError::FetchKlineDataTaskFailed { .. } => 1013,               // fetch kline data task failed
            KlineNodeError::InsufficientBacktestData { .. } => 1014,               // insufficient backtest data for exchange
            KlineNodeError::PendingUpdateKlineNotExist { .. } => 1015,             // pending update kline not exist
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            KlineNodeError::NodeError { source, .. } => source.http_status_code(),
            KlineNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            KlineNodeError::EventCenterError { source, .. } => source.http_status_code(),
            KlineNodeError::KeyError { source, .. } => source.http_status_code(),
            KlineNodeError::RegisterExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => source.http_status_code(),

            // 服务器内部错误
            KlineNodeError::InsufficientBacktestDataForMetaTrader5 { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient meta trader 5 kline data
            KlineNodeError::ExchangeModeNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - exchange mode is not configured
            KlineNodeError::SymbolsIsNotConfiguredSnafu { .. } => StatusCode::BAD_REQUEST, // 400 - symbols is not configured
            KlineNodeError::GetMinIntervalFromStrategyFailed { source, .. } => source.http_status_code(), // 400 - get min interval symbols failed
            KlineNodeError::FirstKlineIsEmpty { .. } => StatusCode::INTERNAL_SERVER_ERROR,                // 500 - first kline is empty
            KlineNodeError::AcquireSemaphoreFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,           // 500 - acquire semaphore failed
            KlineNodeError::FetchKlineDataTaskFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - fetch kline data task failed
            KlineNodeError::InsufficientBacktestData { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient backtest data for exchange
            KlineNodeError::PendingUpdateKlineNotExist { .. } => StatusCode::BAD_REQUEST, // 400 - pending update kline not exist
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::NodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            KlineNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            KlineNodeError::EventCenterError { source, .. } => generate_error_code_chain(source, self.error_code()),
            KlineNodeError::KeyError { source, .. } => generate_error_code_chain(source, self.error_code()),
            KlineNodeError::RegisterExchangeFailed { source, .. }
            | KlineNodeError::LoadKlineFromExchangeFailed { source, .. }
            | KlineNodeError::GetMinIntervalFromStrategyFailed { source, .. } => {
                generate_error_code_chain(source.as_ref(), self.error_code())
            }
            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                KlineNodeError::NodeError { source, .. } => source.error_message(language),
                KlineNodeError::NodeStateMachineError { source, .. } => source.error_message(language),
                KlineNodeError::EventCenterError { source, .. } => source.error_message(language),
                KlineNodeError::KeyError { source, .. } => source.error_message(language),
                KlineNodeError::RegisterExchangeFailed { node_name, .. } => {
                    format!("[{}] 注册交易所错误", node_name)
                }
                KlineNodeError::LoadKlineFromExchangeFailed { .. } => {
                    format!("从交易所加载K线历史失败.")
                }
                KlineNodeError::InsufficientBacktestDataForMetaTrader5 {
                    first_kline_datetime,
                    start_time,
                    end_time,
                    ..
                } => {
                    format!(
                        "回测时间范围从{start_time}到{end_time}，但第一根K线的时间为{first_kline_datetime}。 前往Metatrader5终端的工具-> 选项 -> 图表，然后将最大图表数据量设置为Unlimited，然后重启Metatrader5。"
                    )
                }
                KlineNodeError::ExchangeModeNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 交易所模式未配置")
                }
                KlineNodeError::SymbolsIsNotConfiguredSnafu { node_name, .. } => {
                    format!("@[{node_name}] 交易对未配置")
                }
                KlineNodeError::GetMinIntervalFromStrategyFailed { node_name, .. } => {
                    format!("@[{node_name}] 获取最小周期交易对失败.")
                }
                KlineNodeError::FirstKlineIsEmpty {
                    node_name,
                    exchange,
                    symbol,
                    interval,
                    ..
                } => {
                    format!("@[{node_name}] {exchange}-{symbol}-{interval}的第一根K线为空")
                }
                KlineNodeError::AcquireSemaphoreFailed { .. } => {
                    format!("获取并发请求信号量失败.")
                }
                KlineNodeError::FetchKlineDataTaskFailed {
                    node_name,
                    exchange,
                    symbol,
                    interval,
                    ..
                } => {
                    format!("@[{node_name}] 拉取{exchange}-{symbol}-{interval} K线数据任务失败.",)
                }
                KlineNodeError::InsufficientBacktestData {
                    first_kline_datetime,
                    symbol,
                    interval,
                    exchange,
                    start_time,
                    end_time,
                    ..
                } => {
                    format!(
                        "回测时间范围从{start_time}到{end_time}，但{exchange}-{symbol}-{interval}的第一根K线的时间为{first_kline_datetime}"
                    )
                }
                KlineNodeError::PendingUpdateKlineNotExist { symbol, interval, .. } => {
                    format!("待更新K线不存在: {symbol}-{interval}")
                }
            },
        }
    }
}
