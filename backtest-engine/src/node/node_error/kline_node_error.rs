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

    #[snafu(display("strategy error: {source}"))]
    BacktestStrategy {
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] register exchange error"))]
    RegisterExchangeFailed {
        node_name: String,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] get play kline data failed. kline key: [{kline_key}], play index: [{play_index:?}]"))]
    GetPlayKlineDataFailed {
        node_name: String,
        kline_key: String,
        play_index: Option<i32>,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] kline timestamp not equal. kline key: [{kline_key}], play index: [{play_index:?}]"))]
    KlineTimestampNotEqual {
        node_name: String,
        kline_key: String,
        play_index: Option<i32>,
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

    #[snafu(display("@@[{node_name}] data source account is not configured"))]
    DataSourceAccountIsNotConfigured { node_name: NodeName, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] symbols is not configured"))]
    SymbolsIsNotConfigured { node_name: NodeName, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] time range is not configured"))]
    TimeRangeIsNotConfigured { node_name: NodeName, backtrace: Backtrace },

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

    #[snafu(display("@[{node_name}] strategyreturn empty kline. kline key: [{kline_key}], play index: [{play_index:?}]"))]
    ReturnEmptyKline {
        node_name: NodeName,
        kline_key: String,
        play_index: Option<i32>,
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
            KlineNodeError::BacktestStrategy { .. } => 1005,                       // strategy error
            KlineNodeError::RegisterExchangeFailed { .. } => 1006,                 // register exchange failed
            KlineNodeError::GetPlayKlineDataFailed { .. } => 1007,                 // get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => 1008,                 // kline timestamp not equal
            KlineNodeError::LoadKlineFromExchangeFailed { .. } => 1010,            // load kline from exchange failed
            KlineNodeError::InsufficientBacktestDataForMetaTrader5 { .. } => 1011, // insufficient meta trader 5 kline data
            KlineNodeError::DataSourceAccountIsNotConfigured { .. } => 1012,       // data source account is not configured
            KlineNodeError::SymbolsIsNotConfigured { .. } => 1013,                 // selected symbols is not configured
            KlineNodeError::TimeRangeIsNotConfigured { .. } => 1014,               // time range is not configured
            KlineNodeError::GetMinIntervalFromStrategyFailed { .. } => 1015,       // get min interval symbols from strategy failed
            KlineNodeError::FirstKlineIsEmpty { .. } => 1017,                      // first kline is empty
            KlineNodeError::AcquireSemaphoreFailed { .. } => 1018,                 // acquire semaphore failed
            KlineNodeError::FetchKlineDataTaskFailed { .. } => 1019,               // fetch kline data task failed
            KlineNodeError::InsufficientBacktestData { .. } => 1020,               // insufficient backtest data for exchange
            KlineNodeError::ReturnEmptyKline { .. } => 1021,                       // return empty kline
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            KlineNodeError::NodeError { source, .. } => source.http_status_code(),
            KlineNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            KlineNodeError::EventCenterError { source, .. } => source.http_status_code(),
            KlineNodeError::KeyError { source, .. } => source.http_status_code(),
            KlineNodeError::BacktestStrategy { source, .. } => source.http_status_code(),
            KlineNodeError::RegisterExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => source.http_status_code(),

            // 服务器内部错误
            KlineNodeError::GetPlayKlineDataFailed { .. } | // 500 - get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - kline timestamp not equal
            KlineNodeError::InsufficientBacktestDataForMetaTrader5 { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient meta trader 5 kline data
            KlineNodeError::DataSourceAccountIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - data source account is not configured
            KlineNodeError::SymbolsIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - selected symbols is not configured
            KlineNodeError::TimeRangeIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - time range is not configured
            KlineNodeError::GetMinIntervalFromStrategyFailed { source, .. } => source.http_status_code(), // 400 - get min interval symbols failed
            KlineNodeError::FirstKlineIsEmpty { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - first kline is empty
            KlineNodeError::AcquireSemaphoreFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - acquire semaphore failed
            KlineNodeError::FetchKlineDataTaskFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - fetch kline data task failed
            KlineNodeError::InsufficientBacktestData { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient backtest data for exchange
            KlineNodeError::ReturnEmptyKline { .. } => StatusCode::BAD_REQUEST, // 400 - return empty kline
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::NodeError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::EventCenterError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::KeyError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::RegisterExchangeFailed { source, .. }
            | KlineNodeError::LoadKlineFromExchangeFailed { source, .. }
            | KlineNodeError::GetPlayKlineDataFailed { source, .. }
            | KlineNodeError::GetMinIntervalFromStrategyFailed { source, .. }
            | KlineNodeError::BacktestStrategy { source, .. } => generate_error_code_chain(source.as_ref()),

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
                KlineNodeError::BacktestStrategy { source, .. } => source.error_message(language),
                KlineNodeError::RegisterExchangeFailed { node_name, .. } => {
                    format!("[{}] 注册交易所错误", node_name)
                }
                KlineNodeError::GetPlayKlineDataFailed {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("@[{node_name}] 获取K线数据失败，K线键: [{kline_key}]，播放索引: [{:?}]", play_index)
                }
                KlineNodeError::KlineTimestampNotEqual {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("@[{node_name}] K线时间戳不一致，K线键: [{kline_key}]，播放索引: [{:?}]", play_index)
                }
                KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => {
                    format!("从交易所加载K线历史失败。原因: [{}]", source)
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
                KlineNodeError::DataSourceAccountIsNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 数据源账户未配置")
                }
                KlineNodeError::SymbolsIsNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 未配置交易对")
                }
                KlineNodeError::TimeRangeIsNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 未配置回测时间范围")
                }
                KlineNodeError::GetMinIntervalFromStrategyFailed { node_name, source, .. } => {
                    format!("@[{node_name}] 获取最小周期交易对失败。原因: [{}]", source)
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
                KlineNodeError::AcquireSemaphoreFailed { source, .. } => {
                    format!("获取并发请求信号量失败。原因: [{}]", source)
                }
                KlineNodeError::FetchKlineDataTaskFailed {
                    node_name,
                    exchange,
                    symbol,
                    interval,
                    source,
                    ..
                } => {
                    format!(
                        "@[{node_name}] 拉取{exchange}-{symbol}-{interval} K线数据任务失败。原因: [{}]",
                        source
                    )
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
                KlineNodeError::ReturnEmptyKline {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("@[{node_name}] 策略返回空K线。K线键: [{kline_key}]，播放索引: [{:?}]", play_index)
                }
            },
        }
    }
}
