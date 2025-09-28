use crate::error::ErrorCode;
use crate::error::error_trait::{Language, StarRiverErrorTrait};
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {
    #[snafu(display("kline node [{node_name}({node_id})] register exchange error"))]
    RegisterExchangeFailed {
        node_id: String,
        node_name: String,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("kline node config field [{field_name}]'s value is null"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("kline node backtest config deserialization failed. reason: [{source}]"))]
    ConfigDeserializationFailed { source: serde_json::Error, backtrace: Backtrace },

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
    LoadKlineHistoryFromExchangeFailed {
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
    InsufficientKlineData {
        first_kline_datetime: String,
        start_time: String,
        end_time: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for KlineNodeError {
    fn get_prefix(&self) -> &'static str {
        "KLINE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            KlineNodeError::RegisterExchangeFailed { .. } => 1001, // 注册交易所失败
            KlineNodeError::ConfigFieldValueNull { .. } => 1002,   // 配置字段值为空
            KlineNodeError::ConfigDeserializationFailed { .. } => 1003, // 配置反序列化失败
            KlineNodeError::GetPlayKlineDataFailed { .. } => 1004, // 获取播放K线数据失败
            KlineNodeError::KlineTimestampNotEqual { .. } => 1005, // K线时间戳不一致
            KlineNodeError::NoMinIntervalSymbol { .. } => 1006,    // 没有最小周期K线
            KlineNodeError::LoadKlineHistoryFromExchangeFailed { .. } => 1007, // 从交易所加载K线历史失败
            KlineNodeError::InitKlineDataFailed { .. } => 1008,    // 初始化K线数据失败
            KlineNodeError::AppendKlineDataFailed { .. } => 1009,  // 追加K线数据失败
            KlineNodeError::InsufficientKlineData { .. } => 1010,  // 缺乏K线数据
        };

        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            KlineNodeError::RegisterExchangeFailed { .. }
                | KlineNodeError::ConfigFieldValueNull { .. }
                | KlineNodeError::ConfigDeserializationFailed { .. }
                | KlineNodeError::KlineTimestampNotEqual { .. }
                | KlineNodeError::NoMinIntervalSymbol { .. }
                | KlineNodeError::LoadKlineHistoryFromExchangeFailed { .. }
                | KlineNodeError::InitKlineDataFailed { .. }
                | KlineNodeError::AppendKlineDataFailed { .. }
                | KlineNodeError::InsufficientKlineData { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All KlineNodeError variants have no source or external sources
        // that don't implement our trait (serde_json::Error)

        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::RegisterExchangeFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            // For errors with external sources or no source
            KlineNodeError::ConfigFieldValueNull { .. } |
            // For errors with external sources that don't implement our trait
            KlineNodeError::ConfigDeserializationFailed { .. } |
            KlineNodeError::GetPlayKlineDataFailed { .. } |
            KlineNodeError::KlineTimestampNotEqual { .. } |
            KlineNodeError::NoMinIntervalSymbol { .. } => vec![self.error_code()],
            KlineNodeError::LoadKlineHistoryFromExchangeFailed { source, .. } |
            KlineNodeError::InitKlineDataFailed { source, .. } |
            KlineNodeError::AppendKlineDataFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            KlineNodeError::InsufficientKlineData { .. } => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                KlineNodeError::RegisterExchangeFailed { node_name, node_id, .. } => {
                    format!("K线节点 [{}({})] 注册交易所错误", node_name, node_id)
                }
                KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("K线节点配置字段 [{}] 值为空", field_name)
                }
                KlineNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("K线节点回测配置反序列化失败，原因: [{}]", source)
                }
                KlineNodeError::GetPlayKlineDataFailed {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("K线节点 [{node_name}] 获取K线数据失败，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::KlineTimestampNotEqual {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("K线节点 [{node_name}] K线时间戳不一致，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::NoMinIntervalSymbol { symbol, .. } => {
                    format!("K线节点没有找到最小周期K线，交易对: [{symbol}]")
                }
                KlineNodeError::LoadKlineHistoryFromExchangeFailed { source, .. } => {
                    format!("从交易所加载K线历史失败，原因: [{}]", source)
                }
                KlineNodeError::InitKlineDataFailed { source, .. } => {
                    format!("初始化K线数据失败，原因: [{}]", source)
                }
                KlineNodeError::AppendKlineDataFailed { source, .. } => {
                    format!("追加K线数据失败，原因: [{}]", source)
                }
                KlineNodeError::InsufficientKlineData {
                    first_kline_datetime,
                    start_time,
                    end_time,
                    ..
                } => {
                    format!(
                        "回测时间范围从{start_time}到{end_time}，但第一根K线的时间为{first_kline_datetime}。 前往Metatrader5终端的工具-> 选项 -> 图表，然后将最大图表数据量设置为Unlimited，然后重启Metatrader5。"
                    )
                }
            },
        }
    }
}
