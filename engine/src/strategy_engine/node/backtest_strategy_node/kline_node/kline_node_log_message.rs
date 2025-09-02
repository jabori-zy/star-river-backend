use serde::{Serialize, Deserialize};
use types::market::Exchange;
use crate::log_message;
use crate::strategy_engine::log_message::*;



log_message!(
    NodeStateLogMsg,
    params: (
        node_id: String,
        node_name: String,
        node_state: String,
    ),
    en: "Node [{node_name}({node_id})] current state is: {node_state}",
    zh: "{node_name} ({node_id}) 当前状态是: {node_state}"
);


log_message!(
    ListenExternalEventsMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] starting to listen external events",
    zh: "{node_name} ({node_id}) 开始监听外部事件"
);

log_message!(
    ListenNodeEventsMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] starting to listen other node events",
    zh: "{node_name} ({node_id}) 开始监听其他节点事件"
);

log_message!(
    ListenStrategyInnerEventsMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] starting to listen strategy inner events",
    zh: "{node_name} ({node_id}) 开始监听策略内部事件"
);

log_message!(
    ListenStrategyCommandMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] starting to listen strategy command",
    zh: "{node_name} ({node_id}) 开始监听策略命令"
);




// KlineNode 的具体日志消息实现
log_message!(
    StartRegisterExchangeMsg,
    params: (
        node_id: String,
        node_name: String,
        exchange: Exchange,
        account_id: i32
    ),
    en: "Node [{node_name}({node_id})] starting to register exchange [{exchange}] with account [{account_id}]",
    zh: "{node_name} ({node_id}) 开始注册交易所: {exchange} (账户: {account_id})"
);

log_message!(
    RegisterExchangeSuccessMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] exchange registration successful",
    zh: "{node_name} ({node_id}) 交易所注册成功"
);

log_message!(
    RegisterExchangeFailedMsg,
    params: (
        node_id: String,
        node_name: String,
        error: String
    ),
    en: "Node [{node_name}({node_id})] exchange registration failed: {error}",
    zh: "{node_name} ({node_id}) 交易所注册失败: {error}"
);

log_message!(
    StartLoadKlineDataMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] starting to load kline data from exchange",
    zh: "{node_name} ({node_id}) 开始从交易所加载K线数据"
);

log_message!(
    LoadKlineDataSuccessMsg,
    params: (
        node_id: String,
        node_name: String
    ),
    en: "Node [{node_name}({node_id})] kline data loading successful",
    zh: "{node_name} ({node_id}) K线数据加载成功"
);

log_message!(
    LoadKlineDataFailedMsg,
    params: (
        node_id: String,
        node_name: String,
        error: String
    ),
    en: "Node [{node_name}({node_id})] kline data loading failed: {error}",
    zh: "{node_name} ({node_id}) K线数据加载失败: {error}"
);

log_message!(
    ProcessKlineSignalMsg,
    params: (
        node_id: String,
        node_name: String,
        signal_index: i32,
        play_index: i32
    ),
    en: "Node [{node_name}({node_id})] received kline play signal, signal index: {signal_index}, node index: {play_index}",
    zh: "{node_name} ({node_id}) 接收到K线播放信号，信号索引: {signal_index}, 节点索引: {play_index}"
);

log_message!(
    KlineIndexMismatchMsg,
    params: (
        node_id: String,
        node_name: String,
        cache_index: i32,
        signal_index: i32
    ),
    en: "Node [{node_name}({node_id})] kline cache index mismatch - cache index: {cache_index}, signal index: {signal_index}",
    zh: "{node_name}({node_id}) K线缓存索引不匹配 - 缓存索引: {cache_index}, 信号索引: {signal_index}"
);

log_message!(
    GetKlineCacheFailedMsg,
    params: (
        node_id: String,
        node_name: String,
        symbol: String,
        error: String
    ),
    en: "Node [{node_name}({node_id})] failed to get history kline cache - Symbol: {symbol}, Error: {error}",
    zh: "{node_name} ({node_id}) 获取历史K线缓存失败 - 交易对: {symbol}, 错误: {error}"
);

log_message!(
    SendKlineEventSuccessMsg,
    params: (
        node_id: String,
        node_name: String,
        symbol: String
    ),
    en: "Node [{node_name}({node_id})] kline event sent successfully - Symbol: {symbol}",
    zh: "{node_name} ({node_id}) K线事件发送成功 - 交易对: {symbol}"
);

log_message!(
    SendKlineEventFailedMsg,
    params: (
        node_id: String,
        node_name: String,
        symbol: String,
        error: String
    ),
    en: "Node [{node_name}({node_id})] kline event sending failed - Symbol: {symbol}, Error: {error}",
    zh: "{node_name} ({node_id}) K线事件发送失败 - 交易对: {symbol}, 错误: {error}"
);