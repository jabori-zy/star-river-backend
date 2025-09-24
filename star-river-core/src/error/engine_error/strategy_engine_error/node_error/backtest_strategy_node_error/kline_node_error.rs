use crate::error::ErrorCode;
use crate::error::error_trait::{Language, StarRiverErrorTrait};
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {
    #[snafu(display("kline node [{node_name}({node_id})] register exchange error"))]
    RegisterExchange {
        node_id: String,
        node_name: String,
        #[snafu(source)]
        source: Arc<dyn StarRiverErrorTrait>,
        backtrace: Backtrace,
    },

    #[snafu(display("kline node config field [{field_name}]'s value is null"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("kline node backtest config deserialization failed. reason: [{source}]"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("[{node_name}] get kline data failed. kline key: [{kline_key}], play index: [{play_index}]"))]
    GetKlineData {
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
    NoMinIntervalSymbol {
        symbol: String,
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
            KlineNodeError::RegisterExchange { .. } => 1001,
            KlineNodeError::ConfigFieldValueNull { .. } => 1002,
            KlineNodeError::ConfigDeserializationFailed { .. } => 1003,
            KlineNodeError::GetKlineData { .. } => 1004,
            KlineNodeError::KlineTimestampNotEqual { .. } => 1005,
            KlineNodeError::NoMinIntervalSymbol { .. } => 1006,
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
            KlineNodeError::RegisterExchange { .. }
                | KlineNodeError::ConfigFieldValueNull { .. }
                | KlineNodeError::ConfigDeserializationFailed { .. }
                | KlineNodeError::KlineTimestampNotEqual { .. }
                | KlineNodeError::NoMinIntervalSymbol { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All KlineNodeError variants have no source or external sources
        // that don't implement our trait (serde_json::Error)

        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::RegisterExchange { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            // For errors with external sources or no source
            KlineNodeError::ConfigFieldValueNull { .. } |
            // For errors with external sources that don't implement our trait
            KlineNodeError::ConfigDeserializationFailed { .. } |
            KlineNodeError::GetKlineData { .. } |
            KlineNodeError::KlineTimestampNotEqual { .. } |
            KlineNodeError::NoMinIntervalSymbol { .. } => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                KlineNodeError::RegisterExchange { node_name, node_id, .. } => {
                    format!("K线节点 [{}({})] 注册交易所错误", node_name, node_id)
                }
                KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("K线节点配置字段 [{}] 值为空", field_name)
                }
                KlineNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("K线节点回测配置反序列化失败，原因: [{}]", source)
                }
                KlineNodeError::GetKlineData {
                    node_name,
                    kline_key,
                    play_index,
                    ..
                } => {
                    format!("K线节点 [{node_name}] 获取K线数据失败，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::KlineTimestampNotEqual { node_name, kline_key, play_index, .. } => {
                    format!("K线节点 [{node_name}] K线时间戳不一致，K线键: [{kline_key}]，播放索引: [{play_index}]")
                }
                KlineNodeError::NoMinIntervalSymbol { symbol, .. } => {
                    format!("K线节点没有找到最小周期K线，交易对: [{symbol}]")
                }
            },
        }
    }
}
