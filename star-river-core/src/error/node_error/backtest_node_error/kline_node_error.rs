use crate::error::ErrorCode;
use crate::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {
    #[snafu(display("[{node_name}] register exchange error"))]
    RegisterExchangeFailed {
        node_id: String,
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
    InsufficientKlineData {
        first_kline_datetime: String,
        start_time: String,
        end_time: String,
        backtrace: Backtrace,
    },


    #[snafu(display("kline node [{node_id}] name is null"))]
    NodeNameIsNull {
        node_id: String,
        backtrace: Backtrace,
    },

    #[snafu(display("kline node id is null"))]
    NodeIdIsNull {
        backtrace: Backtrace,
    },

    #[snafu(display("kline node [{node_id}] data is null"))]
    NodeDataIsNull {
        node_id: String,
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
            KlineNodeError::RegisterExchangeFailed { .. } => 1001, // 注册交易所失败
            KlineNodeError::GetPlayKlineDataFailed { .. } => 1002, // 获取播放K线数据失败
            KlineNodeError::KlineTimestampNotEqual { .. } => 1003, // K线时间戳不一致
            KlineNodeError::NoMinIntervalSymbol { .. } => 1004,    // 没有最小周期K线
            KlineNodeError::LoadKlineFromExchangeFailed { .. } => 1005, // 从交易所加载K线历史失败
            KlineNodeError::InitKlineDataFailed { .. } => 1006,    // 初始化K线数据失败
            KlineNodeError::AppendKlineDataFailed { .. } => 1007,  // 追加K线数据失败
            KlineNodeError::InsufficientKlineData { .. } => 1008,  // 缺乏K线数据
            KlineNodeError::NodeNameIsNull { .. } => 1009,  // 节点名称不能为空
            KlineNodeError::NodeIdIsNull { .. } => 1010,  // 节点ID为空
            KlineNodeError::NodeDataIsNull { .. } => 1011,  // 节点数据为空
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
                | KlineNodeError::KlineTimestampNotEqual { .. }
                | KlineNodeError::NoMinIntervalSymbol { .. }
                | KlineNodeError::LoadKlineFromExchangeFailed { .. }
                | KlineNodeError::InitKlineDataFailed { .. }
                | KlineNodeError::AppendKlineDataFailed { .. }
                | KlineNodeError::InsufficientKlineData { .. }
                | KlineNodeError::NodeNameIsNull { .. }
                | KlineNodeError::NodeIdIsNull { .. }
                | KlineNodeError::NodeDataIsNull { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            KlineNodeError::RegisterExchangeFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            // For errors with external sources or no source
            KlineNodeError::GetPlayKlineDataFailed { .. } |
            KlineNodeError::KlineTimestampNotEqual { .. } |
            KlineNodeError::NoMinIntervalSymbol { .. } => vec![self.error_code()],
            KlineNodeError::LoadKlineFromExchangeFailed { source, .. } |
            KlineNodeError::InitKlineDataFailed { source, .. } |
            KlineNodeError::AppendKlineDataFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            KlineNodeError::InsufficientKlineData { .. } => vec![self.error_code()],
            KlineNodeError::NodeNameIsNull { .. } => vec![self.error_code()],
            KlineNodeError::NodeIdIsNull { .. } => vec![self.error_code()],
            KlineNodeError::NodeDataIsNull { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                KlineNodeError::RegisterExchangeFailed { node_name, node_id, .. } => {
                    format!("K线节点 [{}({})] 注册交易所错误", node_name, node_id)
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
                KlineNodeError::LoadKlineFromExchangeFailed { source, .. } => {
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
                KlineNodeError::NodeNameIsNull { node_id, .. } => {
                    format!("K线节点 [{node_id}] 名称为空")
                }
                KlineNodeError::NodeIdIsNull { .. } => {
                    format!("K线节点 ID为空")
                }
                KlineNodeError::NodeDataIsNull { node_id, .. } => {
                    format!("K线节点 [{node_id}] 数据为空")
                }
            },
        }
    }
}
