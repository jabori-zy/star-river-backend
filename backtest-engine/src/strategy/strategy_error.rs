use std::sync::Arc;

use database::error::DatabaseError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};
use strategy_core::error::{StrategyError, strategy_state_machine_error::StrategyStateMachineError};

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
            BacktestStrategyError::StrategyError { .. } => 1001,              // 策略错误.
            BacktestStrategyError::StrategyStateMachineError { .. } => 1002,  // 策略状态机错误
            BacktestStrategyError::BacktestNodeError { .. } => 1003,          // 节点错误
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => 1004, // 更新策略状态失败
            BacktestStrategyError::PlayFinished { .. } => 1005,               // 所有回测数据播放完毕
            BacktestStrategyError::AlreadyPlaying { .. } => 1006,             // 策略正在播放，无法再次播放
            BacktestStrategyError::AlreadyPausing { .. } => 1007,             // 策略正在暂停，无法再次暂停
            BacktestStrategyError::IntervalNotSame { .. } => 1008,            // 不同symbol的最小周期不相同
            BacktestStrategyError::GetDataFailed { .. } => 1009,              // 获取数据失败
            BacktestStrategyError::GetDataByDatetimeFailed { .. } => 1010,    // 获取数据失败
            BacktestStrategyError::GetStartNodeConfigFailed { .. } => 1011,   // 获取开始节点配置失败
            BacktestStrategyError::KlineDataLengthNotSame { .. } => 1012,     // kline数据长度不相同
            BacktestStrategyError::KeyNotFound { .. } => 1013,                // kline key未找到
            BacktestStrategyError::PlayIndexOutOfRange { .. } => 1014,        // 播放索引超出范围
            BacktestStrategyError::GetNodeConfigFailed { .. } => 1015,        // 获取节点配置失败
            BacktestStrategyError::MissingDataSource { .. } => 1016,          // 缺少数据源
            BacktestStrategyError::MissingStartNode { .. } => 1017,           // 缺少开始节点
            BacktestStrategyError::SymbolIsNotMinInterval { .. } => 1018,     // kline key 不是最小周期symbol
            BacktestStrategyError::NoSymbolConfigured { .. } => 1019,         // 没有symbol配置
            BacktestStrategyError::TimeRangeNotConfigured { .. } => 1020,     // 时间范围未配置
        };
        format!("{prefix}_{code:04}")
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            // Transparent errors - delegate to source
            BacktestStrategyError::StrategyError { source, .. } => source.http_status_code(),
            BacktestStrategyError::StrategyStateMachineError { source, .. } => source.http_status_code(),
            BacktestStrategyError::BacktestNodeError { source, .. } => source.http_status_code(),

            // 服务器内部错误 (500)
            BacktestStrategyError::GetDataFailed { .. }
            | BacktestStrategyError::GetDataByDatetimeFailed { .. }
            | BacktestStrategyError::KlineDataLengthNotSame { .. }
            | BacktestStrategyError::PlayIndexOutOfRange { .. }
            | BacktestStrategyError::GetNodeConfigFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // 客户端错误 - 配置/数据问题 (400)
            BacktestStrategyError::GetStartNodeConfigFailed { .. } | BacktestStrategyError::IntervalNotSame { .. } => {
                StatusCode::BAD_REQUEST
            }

            // 客户端错误 - 资源未找到 (404)
            BacktestStrategyError::KeyNotFound { .. } => StatusCode::NOT_FOUND,

            // 客户端错误 - 冲突/状态错误 (409)
            BacktestStrategyError::AlreadyPlaying { .. } | BacktestStrategyError::AlreadyPausing { .. } => StatusCode::CONFLICT,

            // 成功但已完成 (200 - 虽然是错误但在业务上是正常完成)
            BacktestStrategyError::PlayFinished { .. } => StatusCode::OK,

            // 服务不可用 (503)
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,

            // 客户端错误 - 配置/数据问题 (400)
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
            BacktestStrategyError::StrategyError { source, .. } => generate_error_code_chain(source),
            BacktestStrategyError::StrategyStateMachineError { source, .. } => generate_error_code_chain(source),
            BacktestStrategyError::BacktestNodeError { source, .. } => generate_error_code_chain(source),

            // Non-transparent errors - return own error code
            _ => vec![self.error_code()],
        }
    }
}
