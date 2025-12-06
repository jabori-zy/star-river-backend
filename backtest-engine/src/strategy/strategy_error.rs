use std::sync::Arc;

use database::error::DatabaseError;
use event_center::EventCenterError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};
use strategy_core::error::{StrategyError, strategy_state_machine_error::StrategyStateMachineError};
use virtual_trading::error::VtsError;

use crate::node::node_error::BacktestNodeError;
// use event_center::EventCenterError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyError {
    #[snafu(transparent)]
    StrategyError { source: StrategyError, backtrace: Backtrace },

    #[snafu(transparent)]
    StrategyStateMachineError {
        source: StrategyStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    BacktestNodeError { source: BacktestNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    EventCenterError { source: EventCenterError, backtrace: Backtrace },

    #[snafu(transparent)]
    VtsError { source: VtsError, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] update status failed: {source}"))]
    UpdateStrategyStatusFailed {
        strategy_name: String,
        source: DatabaseError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] all backtest data played finished"))]
    PlayFinished { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("already playing, cannot play again"))]
    AlreadyPlaying { backtrace: Backtrace },

    #[snafu(display("already pausing, cannot pause again"))]
    AlreadyPausing { backtrace: Backtrace },

    #[snafu(display("different symbols have different minimum intervals: {symbols:?}"))]
    IntervalNotSame {
        symbols: Vec<(String, String)>,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get data failed. key: {key}, datetime: {datetime:?}"))]
    GetDataFailed {
        strategy_name: String,
        key: String,
        datetime: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get data by timestamp failed. key: {key}, datetime: {datetime}"))]
    GetDataByDatetimeFailed {
        strategy_name: String,
        key: String,
        datetime: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get start node config failed"))]
    GetStartNodeConfigFailed {
        strategy_name: String,
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("#[{strategy_name}] get node @[{node_name}] config failed"))]
    GetNodeConfigFailed {
        strategy_name: String,
        node_name: NodeName,
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] kline data lengths are not all the same"))]
    KlineDataLengthNotSame { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] key not found: {key}"))]
    KeyNotFound {
        strategy_name: String,
        key: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] play index out of range, kline data length: {kline_data_length}, play index: {play_index}"))]
    PlayIndexOutOfRange {
        strategy_name: String,
        kline_data_length: u32,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("#[{strategy_name}] missing data source. try to add a kline node to get data"))]
    MissingDataSource { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("#[{strategy_name}] missing start node"))]
    MissingStartNode { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("#[{strategy_name}] {symbol}-{interval} is not min interval symbol"))]
    SymbolIsNotMinInterval {
        strategy_name: String,
        symbol: String,
        interval: String,
        backtrace: Backtrace,
    },

    #[snafu(display("#[{strategy_name}] no symbol configured"))]
    NoSymbolConfigured { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("#[{strategy_name}] time range not configured"))]
    TimeRangeNotConfigured { strategy_name: String, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for BacktestStrategyError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_STRATEGY"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            BacktestStrategyError::StrategyError { .. } => 1001,              // Strategy error
            BacktestStrategyError::StrategyStateMachineError { .. } => 1002,  // Strategy state machine error
            BacktestStrategyError::BacktestNodeError { .. } => 1003,          // Node error
            BacktestStrategyError::EventCenterError { .. } => 1004,           // Event center error
            BacktestStrategyError::VtsError { .. } => 1005,                   // Virtual trading system error
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => 1006, // Update strategy status failed
            BacktestStrategyError::PlayFinished { .. } => 1007,               // All backtest data playback finished
            BacktestStrategyError::AlreadyPlaying { .. } => 1008,             // Strategy is already playing, cannot play again
            BacktestStrategyError::AlreadyPausing { .. } => 1009,             // Strategy is already pausing, cannot pause again
            BacktestStrategyError::IntervalNotSame { .. } => 1010,            // Minimum interval of different symbols not the same
            BacktestStrategyError::GetDataFailed { .. } => 1011,              // Get data failed
            BacktestStrategyError::GetDataByDatetimeFailed { .. } => 1012,    // Get data by datetime failed
            BacktestStrategyError::GetStartNodeConfigFailed { .. } => 1013,   // Get start node config failed
            BacktestStrategyError::KlineDataLengthNotSame { .. } => 1014,     // Kline data length not the same
            BacktestStrategyError::KeyNotFound { .. } => 1015,                // Kline key not found
            BacktestStrategyError::PlayIndexOutOfRange { .. } => 1016,        // Play index out of range
            BacktestStrategyError::GetNodeConfigFailed { .. } => 1017,        // Get node config failed
            BacktestStrategyError::MissingDataSource { .. } => 1018,          // Missing data source
            BacktestStrategyError::MissingStartNode { .. } => 1019,           // Missing start node
            BacktestStrategyError::SymbolIsNotMinInterval { .. } => 1020,     // Kline key is not minimum interval symbol
            BacktestStrategyError::NoSymbolConfigured { .. } => 1021,         // No symbol configured
            BacktestStrategyError::TimeRangeNotConfigured { .. } => 1022,     // Time range not configured
        };
        format!("{prefix}_{code:04}")
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            // Transparent errors - delegate to source
            BacktestStrategyError::StrategyError { source, .. } => source.http_status_code(),
            BacktestStrategyError::StrategyStateMachineError { source, .. } => source.http_status_code(),
            BacktestStrategyError::BacktestNodeError { source, .. } => source.http_status_code(),
            BacktestStrategyError::EventCenterError { source, .. } => source.http_status_code(),
            BacktestStrategyError::VtsError { source, .. } => source.http_status_code(),
            // Server internal error (500)
            BacktestStrategyError::GetDataFailed { .. }
            | BacktestStrategyError::GetDataByDatetimeFailed { .. }
            | BacktestStrategyError::KlineDataLengthNotSame { .. }
            | BacktestStrategyError::PlayIndexOutOfRange { .. }
            | BacktestStrategyError::GetNodeConfigFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // Client error - configuration/data issues (400)
            BacktestStrategyError::GetStartNodeConfigFailed { .. } | BacktestStrategyError::IntervalNotSame { .. } => {
                StatusCode::BAD_REQUEST
            }

            // Client error - resource not found (404)
            BacktestStrategyError::KeyNotFound { .. } => StatusCode::NOT_FOUND,

            // Client error - conflict/state error (409)
            BacktestStrategyError::AlreadyPlaying { .. } | BacktestStrategyError::AlreadyPausing { .. } => StatusCode::CONFLICT,

            // Success but completed (200 - although an error, it's a normal completion in business terms)
            BacktestStrategyError::PlayFinished { .. } => StatusCode::OK,

            // Service unavailable (503)
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,

            // Client error - configuration/data issues (400)
            BacktestStrategyError::MissingDataSource { .. }
            | BacktestStrategyError::TimeRangeNotConfigured { .. }
            | BacktestStrategyError::MissingStartNode { .. }
            | BacktestStrategyError::SymbolIsNotMinInterval { .. }
            | BacktestStrategyError::NoSymbolConfigured { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // Transparent errors - delegate to source
                BacktestStrategyError::StrategyError { source, .. } => source.error_message(language),
                BacktestStrategyError::StrategyStateMachineError { source, .. } => source.error_message(language),
                BacktestStrategyError::BacktestNodeError { source, .. } => source.error_message(language),
                BacktestStrategyError::EventCenterError { source, .. } => source.error_message(language),
                BacktestStrategyError::VtsError { source, .. } => source.error_message(language),
                BacktestStrategyError::UpdateStrategyStatusFailed { strategy_name, source, .. } => {
                    format!("策略 [{strategy_name}] 更新状态失败: {source}")
                }
                BacktestStrategyError::PlayFinished { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] 所有数据播放完毕")
                }
                BacktestStrategyError::AlreadyPlaying { .. } => {
                    format!("策略正在播放，无法再次播放")
                }
                BacktestStrategyError::AlreadyPausing { .. } => {
                    format!("策略正在暂停，无法再次暂停")
                }
                BacktestStrategyError::IntervalNotSame { symbols, .. } => {
                    format!("不同交易对的最小周期不相同: {symbols:?}")
                }
                BacktestStrategyError::GetDataFailed {
                    strategy_name,
                    key,
                    datetime,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 获取数据失败: 数据键: {key}, 时间: {datetime:?}")
                }
                BacktestStrategyError::GetDataByDatetimeFailed {
                    strategy_name,
                    key,
                    datetime,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 获取数据失败. 数据键: {key}, 时间: {datetime}")
                }
                BacktestStrategyError::GetStartNodeConfigFailed { strategy_name, .. } => {
                    format!("[{strategy_name}] 获取开始节点配置失败")
                }
                BacktestStrategyError::KlineDataLengthNotSame { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] kline数据长度不相同")
                }
                BacktestStrategyError::KeyNotFound {
                    strategy_name,
                    key: kline_key,
                    ..
                } => {
                    format!("策略 [{strategy_name}] kline key 不存在: {kline_key}")
                }
                BacktestStrategyError::PlayIndexOutOfRange {
                    strategy_name,
                    kline_data_length,
                    play_index,
                    ..
                } => {
                    format!("#[{strategy_name}] 播放索引超出范围: k线数据长度: {kline_data_length}, 播放索引: {play_index}")
                }
                BacktestStrategyError::GetNodeConfigFailed {
                    strategy_name, node_name, ..
                } => {
                    format!("#[{strategy_name}] 获取节点 @[{node_name}] 配置失败")
                }
                BacktestStrategyError::MissingDataSource { strategy_name, .. } => {
                    format!("#[{strategy_name}] 缺少数据源. 尝试添加一个k线节点来获取数据")
                }
                BacktestStrategyError::MissingStartNode { strategy_name, .. } => {
                    format!("#[{strategy_name}] 缺少开始节点")
                }
                BacktestStrategyError::SymbolIsNotMinInterval {
                    strategy_name,
                    symbol,
                    interval,
                    ..
                } => {
                    format!("#[{strategy_name}] {symbol}-{interval} 不是最小周期交易对")
                }
                BacktestStrategyError::NoSymbolConfigured { strategy_name, .. } => {
                    format!("#[{strategy_name}] 未配置交易对")
                }
                BacktestStrategyError::TimeRangeNotConfigured { strategy_name, .. } => {
                    format!("#[{strategy_name}] 回测时间范围未配置")
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // Transparent errors - delegate to source
            BacktestStrategyError::StrategyError { source, .. } => generate_error_code_chain(source, self.error_code()),
            BacktestStrategyError::StrategyStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            BacktestStrategyError::BacktestNodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            BacktestStrategyError::EventCenterError { source, .. } => generate_error_code_chain(source, self.error_code()),
            BacktestStrategyError::VtsError { source, .. } => generate_error_code_chain(source, self.error_code()),
            // Non-transparent errors - return own error code
            _ => vec![self.error_code()],
        }
    }
}
