use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode, generate_error_code_chain};
use snafu::{Backtrace, Snafu};
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {
    #[snafu(display("[{node_name}] register exchange error"))]
    RegisterExchangeFailed {
        node_name: String,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("[{node_name}] get play kline data failed. kline key: [{kline_key}], play index: [{play_index}]"))]
    GetPlayKlineDataFailed {
        node_name: String,
        kline_key: String,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("[{node_name}] kline timestamp not equal. kline key: [{kline_key}], play index: [{play_index}]"))]
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
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for KlineNodeError {
    fn get_prefix(&self) -> &'static str {
        "KLINE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            KlineNodeError::RegisterExchangeFailed { .. } => 1001, // register exchange failed
            KlineNodeError::GetPlayKlineDataFailed { .. } => 1002, // get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => 1003, // kline timestamp not equal
            KlineNodeError::NoMinIntervalSymbol { .. } => 1004,    // no min interval symbol found
            KlineNodeError::LoadKlineFromExchangeFailed { .. } => 1005, // load kline from exchange failed
            KlineNodeError::InitKlineDataFailed { .. } => 1006,    // init kline data failed
            KlineNodeError::AppendKlineDataFailed { .. } => 1007,  // append kline data failed
            KlineNodeError::InsufficientMetaTrader5KlineData { .. } => 1008,  // insufficient meta trader 5 kline data
        };

        format!("{}_{:04}", prefix, code)
    }


    fn http_status_code(&self) -> StatusCode {
        match self {
            // 委托给底层错误源
            KlineNodeError::RegisterExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => source.http_status_code(),
            KlineNodeError::InitKlineDataFailed { source, .. } => source.http_status_code(),
            KlineNodeError::AppendKlineDataFailed { source, .. } => source.http_status_code(),

            // 服务器内部错误
            KlineNodeError::GetPlayKlineDataFailed { .. } | // 500 - get play kline data failed
            KlineNodeError::KlineTimestampNotEqual { .. } => StatusCode::INTERNAL_SERVER_ERROR, // 500 - kline timestamp not equal
            
            KlineNodeError::NoMinIntervalSymbol { .. } => StatusCode::NOT_FOUND, // 404 - no min interval symbol found
            KlineNodeError::InsufficientMetaTrader5KlineData { .. } => StatusCode::BAD_REQUEST, // 400 - insufficient meta trader 5 kline data
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::RegisterExchangeFailed { source, .. } |
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } |
            KlineNodeError::InitKlineDataFailed { source, .. } |
            KlineNodeError::AppendKlineDataFailed { source, .. } => generate_error_code_chain(source.as_ref()),

            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                KlineNodeError::RegisterExchangeFailed { node_name, .. } => {
                    format!("[{}] 注册交易所错误", node_name)
                }
                KlineNodeError::GetPlayKlineDataFailed {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("[{node_name}] 获取K线数据失败，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::KlineTimestampNotEqual {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("[{node_name}] K线时间戳不一致，K线键: [{kline_key}]，播放索引: [{play_index}]")
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
                KlineNodeError::InsufficientMetaTrader5KlineData {first_kline_datetime,start_time,end_time,..} => {
                    format!(
                        "回测时间范围从{start_time}到{end_time}，但第一根K线的时间为{first_kline_datetime}。 前往Metatrader5终端的工具-> 选项 -> 图表，然后将最大图表数据量设置为Unlimited，然后重启Metatrader5。"
                    )
                },
            },
        }
    }
}
