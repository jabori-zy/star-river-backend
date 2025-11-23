use std::sync::Arc;

use event_center::EventCenterError;
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

    #[snafu(display("@[{node_name}] register exchange error"))]
    RegisterExchangeFailed {
        node_name: String,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] get play kline data failed. kline key: [{kline_key}], play index: [{play_index}]"))]
    GetPlayKlineDataFailed {
        node_name: String,
        kline_key: String,
        play_index: u32,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] kline timestamp not equal. kline key: [{kline_key}], play index: [{play_index}]"))]
    KlineTimestampNotEqual {
        node_name: String,
        kline_key: String,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("no min interval symbol found for [{symbol}]"))]
    NoMinIntervalSymbol { symbol: String, backtrace: Backtrace },

    #[snafu(display("load kline history from exchange failed."))]
    LoadKlineFromExchangeFailed {
        exchange: String,
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("init kline data failed."))]
    InitKlineDataFailed {
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("append kline data failed."))]
    AppendKlineDataFailed {
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "the backtest time range is from {start_time} to {end_time}, but the first kline datetime is {first_kline_datetime} Go to Metatrader5 Terminal's Tools -> Options -> Charts and then set Max bars in charts to Unlimited. Then restart Metatrader5. "
    ))]
    InsufficientMetaTrader5KlineData {
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
    GetMinIntervalSymbolsFailed {
        node_name: NodeName,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] no min interval symbol found"))]
    MinIntervalSymbolIsNone { node_name: NodeName, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for KlineNodeError {
    fn get_prefix(&self) -> &'static str {
        "KLINE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            KlineNodeError::NodeError { .. } => 1001,                        // node error
            KlineNodeError::NodeStateMachineError { .. } => 1002,            // node state machine error
            KlineNodeError::EventCenterError { .. } => 1003,                 // event center error
            KlineNodeError::RegisterExchangeFailed { .. } => 1004,           // register exchange failed
            KlineNodeError::GetPlayKlineDataFailed { .. } => 1005,           // get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => 1006,           // kline timestamp not equal
            KlineNodeError::NoMinIntervalSymbol { .. } => 1007,              // no min interval symbol found
            KlineNodeError::LoadKlineFromExchangeFailed { .. } => 1008,      // load kline from exchange failed
            KlineNodeError::InitKlineDataFailed { .. } => 1009,              // init kline data failed
            KlineNodeError::AppendKlineDataFailed { .. } => 1010,            // append kline data failed
            KlineNodeError::InsufficientMetaTrader5KlineData { .. } => 1011, // insufficient meta trader 5 kline data
            KlineNodeError::DataSourceAccountIsNotConfigured { .. } => 1012, // data source account is not configured
            KlineNodeError::SymbolsIsNotConfigured { .. } => 1013,           // selected symbols is not configured
            KlineNodeError::TimeRangeIsNotConfigured { .. } => 1014,         // time range is not configured
            KlineNodeError::GetMinIntervalSymbolsFailed { .. } => 1015,      // get min interval symbols failed
            KlineNodeError::MinIntervalSymbolIsNone { .. } => 1016,          // min interval symbol is none
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            KlineNodeError::NodeError { source, .. } => source.http_status_code(),
            KlineNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            KlineNodeError::EventCenterError { source, .. } => source.http_status_code(),
            KlineNodeError::RegisterExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::InitKlineDataFailed { source, .. } => source.http_status_code(),
            KlineNodeError::AppendKlineDataFailed { source, .. } => source.http_status_code(),

            // 服务器内部错误
            KlineNodeError::GetPlayKlineDataFailed { .. } | // 500 - get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - kline timestamp not equal
            KlineNodeError::NoMinIntervalSymbol { .. } => StatusCode::NOT_FOUND, // 404 - no min interval symbol found
            KlineNodeError::InsufficientMetaTrader5KlineData { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient meta trader 5 kline data
            KlineNodeError::DataSourceAccountIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - data source account is not configured
            KlineNodeError::SymbolsIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - selected symbols is not configured
            KlineNodeError::TimeRangeIsNotConfigured { .. } => StatusCode::BAD_REQUEST, // 400 - time range is not configured
            KlineNodeError::GetMinIntervalSymbolsFailed { source, .. } => source.http_status_code(), // 400 - get min interval symbols failed
            KlineNodeError::MinIntervalSymbolIsNone { .. } => StatusCode::NOT_FOUND, // 404 - min interval symbol is none
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::NodeError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::EventCenterError { source, .. } => generate_error_code_chain(source),
            KlineNodeError::RegisterExchangeFailed { source, .. }
            | KlineNodeError::LoadKlineFromExchangeFailed { source, .. }
            | KlineNodeError::InitKlineDataFailed { source, .. }
            | KlineNodeError::AppendKlineDataFailed { source, .. }
            | KlineNodeError::GetPlayKlineDataFailed { source, .. }
            | KlineNodeError::GetMinIntervalSymbolsFailed { source, .. } => generate_error_code_chain(source.as_ref()),

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
                KlineNodeError::RegisterExchangeFailed { node_name, .. } => {
                    format!("[{}] 注册交易所错误", node_name)
                }
                KlineNodeError::GetPlayKlineDataFailed {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("@[{node_name}] 获取K线数据失败，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::KlineTimestampNotEqual {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("@[{node_name}] K线时间戳不一致，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::NoMinIntervalSymbol { symbol, .. } => {
                    format!("没有找到最小周期K线，交易对: [{symbol}]")
                }
                KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => {
                    format!("从交易所加载K线历史失败。原因: [{}]", source)
                }
                KlineNodeError::InitKlineDataFailed { source, .. } => {
                    format!("初始化K线数据失败。原因: [{}]", source)
                }
                KlineNodeError::AppendKlineDataFailed { source, .. } => {
                    format!("追加K线数据失败。原因: [{}]", source)
                }
                KlineNodeError::InsufficientMetaTrader5KlineData {
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
                KlineNodeError::GetMinIntervalSymbolsFailed { node_name, source, .. } => {
                    format!("@[{node_name}] 获取最小周期交易对失败。原因: [{}]", source)
                }
                KlineNodeError::MinIntervalSymbolIsNone { node_name, .. } => {
                    format!("@[{node_name}] 没有最小周期交易对")
                }
            },
        }
    }
}
