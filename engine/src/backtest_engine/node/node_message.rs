#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// 通用日志消息 - 多个节点共同使用的日志消息
pub mod common_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    // 所有节点都有的通用状态日志
    log_message!(
        NodeStateLogMsg,
        params: (
            node_name: String,
            node_state: String,
        ),
        en: "[{node_name}] current state is: {node_state}",
        zh: "[{node_name}] 当前状态是: {node_state}"
    );

    // 监听其他节点事件 - 大部分节点都有
    log_message!(
        ListenNodeEventsMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] starting to listen other node events",
        zh: "[{node_name}] 开始监听其他节点事件"
    );

    // 监听策略内部事件 - 大部分节点都有
    log_message!(
        ListenStrategyInnerEventsMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] starting to listen strategy inner events",
        zh: "[{node_name}] 开始监听策略内部事件"
    );

    // 监听策略命令 - 大部分节点都有
    log_message!(
        ListenStrategyCommandMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] starting to listen strategy command",
        zh: "[{node_name}] 开始监听策略命令"
    );

    // 监听外部事件 - 部分节点有
    log_message!(
        ListenExternalEventsMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] starting to listen external events",
        zh: "[{node_name}] 开始监听外部事件"
    );

    // 监听虚拟交易系统事件 - 部分节点有
    log_message!(
        ListenVirtualTradingSystemEventMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] starting to listen virtual trading system events",
        zh: "[{node_name}] 开始监听虚拟交易系统事件"
    );

    // 取消异步任务 - 所有节点都有
    log_message!(
        CancelAsyncTaskMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] canceling async tasks",
        zh: "[{node_name}] 取消异步任务"
    );

    // 取消异步任务成功 - 大部分节点有
    log_message!(
        CancelAsyncTaskSuccessMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] async tasks canceled successfully",
        zh: "[{node_name}] 异步任务取消成功"
    );

    // 注册任务消息 - 部分节点有
    log_message!(
        RegisterTaskMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] registering task",
        zh: "[{node_name}] 注册任务"
    );

    // 注册任务成功消息 - 部分节点有
    log_message!(
        RegisterTaskSuccessMsg,
        params: (
            node_name: String,
        ),
        en: "[{node_name}] task registration successful",
        zh: "[{node_name}] 任务注册成功"
    );

    // 事件发送成功消息 - 多个节点有
    log_message!(
        SendEventSuccessMsg,
        params: (
            node_name: String,
            output_handle_id: String,
            event_type: String
        ),
        en: "[{node_name}] event sent successfully - Output: {output_handle_id}, Type: {event_type}",
        zh: "[{node_name}] 事件发送成功 - 输出: {output_handle_id}, 类型: {event_type}"
    );

    // 事件发送失败消息 - 多个节点有
    log_message!(
        SendEventFailedMsg,
        params: (
            output_handle_id: String,
            event_type: String,
            error: String
        ),
        en: "Node [{node_name}] event sending failed - Output: {output_handle_id}, Type: {event_type}, Error: {error}",
        zh: "{node_name}  事件发送失败 - 输出: {output_handle_id}, 类型: {event_type}, 错误: {error}"
    );

    // 获取K线缓存失败消息 - 多个节点有
    log_message!(
        GetKlineCacheFailedMsg,
        params: (
            symbol: String,
            error: String
        ),
        en: "Node [{node_name}] failed to get kline cache - Symbol: {symbol}, Error: {error}",
        zh: "{node_name}  获取K线缓存失败 - 交易对: {symbol}, 错误: {error}"
    );

    // 虚拟交易系统事件监听终止消息 - 部分节点有
    log_message!(
        VirtualTradingSystemEventTerminatedMsg,
        params: (
            node_id: String,
            node_name: String,
        ),
        en: "Node [{node_name}] virtual trading system event monitoring terminated",
        zh: "{node_name}  虚拟交易系统事件监听已终止"
    );

    log_message!(
        GetMinIntervalSymbolsSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] min interval symbols initialization successful",
        zh: "[{node_name}] 最小周期交易对初始化成功"
    );

    log_message!(
        GetMinIntervalSymbolsFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] min interval symbols initialization failed: {error}",
        zh: "[{node_name}] 最小周期交易对初始化失败: {error}"
    );
}

// StartNode 独特的日志消息
pub mod start_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    log_message!(
        ListenPlayIndexChangeMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to listen play index change",
        zh: "[{node_name}] 开始监听播放索引变化"
    );

    log_message!(
        InitVirtualTradingSystemMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] initializing virtual trading system",
        zh: "[{node_name}] 开始初始化虚拟交易系统"
    );

    log_message!(
        InitVirtualTradingSystemSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] virtual trading system initialization successful",
        zh: "[{node_name}] 虚拟交易系统初始化成功"
    );

    log_message!(
        InitVirtualTradingSystemFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] virtual trading system initialization failed: {error}",
        zh: "[{node_name}] 虚拟交易系统初始化失败: {error}"
    );

    log_message!(
        InitStrategyStatsMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] initializing strategy statistics",
        zh: "[{node_name}] 开始初始化策略统计"
    );

    log_message!(
        InitStrategyStatsSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] strategy statistics initialization successful",
        zh: "[{node_name}] 策略统计初始化成功"
    );

    log_message!(
        InitStrategyStatsFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] strategy statistics initialization failed: {error}",
        zh: "[{node_name}] 策略统计初始化失败: {error}"
    );

    log_message!(
        InitCustomVariableMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] initializing custom variables",
        zh: "[{node_name}] 开始初始化策略统计"
    );

    log_message!(
        InitCustomVariableSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] custom variables initialization successful",
        zh: "[{node_name}] 策略统计初始化成功"
    );

    log_message!(
        InitCustomVariableFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] custom variables initialization failed: {error}",
        zh: "[{node_name}] 策略统计初始化失败: {error}"
    );

    log_message!(
        HandlePlayIndexMsg,
        params: (
            node_name: String,
            play_index: i32
        ),
        en: "[{node_name}] handling play index change: {play_index}",
        zh: "[{node_name}] 处理播放索引变化: {play_index}"
    );

    log_message!(
        SendFinishSignalMsg,
        params: (
            node_name: String,
            signal_index: i32
        ),
        en: "[{node_name}] sending finish signal: {signal_index}",
        zh: "[{node_name}] 发送完成信号: {signal_index}"
    );

    log_message!(
        SendFinishSignalSuccessMsg,
        params: (
            node_id: String,
            node_name: String,
            signal_index: i32
        ),
        en: "Start Node [{node_name}] finish signal sent successfully: {signal_index}",
        zh: "{node_name}  完成信号发送成功: {signal_index}"
    );

    log_message!(
        SendFinishSignalFailedMsg,
        params: (
            node_id: String,
            node_name: String,
            signal_index: i32,
            error: String
        ),
        en: "Start Node [{node_name}] finish signal sending failed: {signal_index}, Error: {error}",
        zh: "{node_name}  完成信号发送失败: {signal_index}, 错误: {error}"
    );
}

// IndicatorNode 独特的日志消息
pub mod indicator_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    log_message!(
        RegisterIndicatorCacheKeyMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to register indicator cache keys",
        zh: "[{node_name}] 开始注册指标缓存键"
    );

    log_message!(
        RegisterIndicatorCacheKeySuccessMsg,
        params: (
            node_name: String,
            indicator_count: usize
        ),
        en: "[{node_name}] indicator cache keys registration successful, registered {indicator_count} indicators",
        zh: "[{node_name}] 指标缓存键注册成功，已注册 {indicator_count} 个指标"
    );

    log_message!(
        RegisterIndicatorCacheKeyFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] indicator cache keys registration failed: {error}",
        zh: "[{node_name}] 指标缓存键注册失败: {error}"
    );

    log_message!(
        CalculateIndicatorMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] start to calculate indicators",
        zh: "[{node_name}] 开始计算指标"
    );

    log_message!(
        CalculateIndicatorSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] indicator calculation successful",
        zh: "[{node_name}] 指标计算成功"
    );

    log_message!(
        CalculateIndicatorFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] indicator calculation failed: {error}",
        zh: "[{node_name}] 指标计算失败: {error}"
    );

    log_message!(
        IndicatorCacheKeyRegisteredMsg,
        params: (
            node_name: String,
            indicator_type: String,
            cache_key: String
        ),
        en: "[{node_name}] indicator cache key registered - Type: {indicator_type}, Key: {cache_key}",
        zh: "[{node_name}] 指标缓存键已注册 - 类型: {indicator_type}, 键: {cache_key}"
    );

    log_message!(
        IndicatorCalculatedMsg,
        params: (
            node_name: String,
            indicator_type: String,
            data_points: usize
        ),
        en: "[{node_name}] indicator calculated successfully - Type: {indicator_type}, Data points: {data_points}",
        zh: "[{node_name}] 指标计算成功 - 类型: {indicator_type}, 数据点: {data_points}"
    );

    log_message!(
        IndicatorCalculationErrorMsg,
        params: (
            node_name: String,
            indicator_type: String,
            error: String
        ),
        en: "[{node_name}] indicator calculation error - Type: {indicator_type}, Error: {error}",
        zh: "[{node_name}] 指标计算错误 - 类型: {indicator_type}, 错误: {error}"
    );

    log_message!(
        SendIndicatorEventSuccessMsg,
        params: (
            node_name: String,
            indicator_type: String,
            output_handle_id: String
        ),
        en: "[{node_name}] indicator event sent successfully - Type: {indicator_type}, Output: {output_handle_id}",
        zh: "[{node_name}] 指标事件发送成功 - 类型: {indicator_type}, 输出: {output_handle_id}"
    );

    log_message!(
        SendIndicatorEventFailedMsg,
        params: (
            node_name: String,
            indicator_type: String,
            output_handle_id: String,
            error: String
        ),
        en: "[{node_name}] indicator event sending failed - Type: {indicator_type}, Output: {output_handle_id}, Error: {error}",
        zh: "[{node_name}] 指标事件发送失败 - 类型: {indicator_type}, 输出: {output_handle_id}, 错误: {error}"
    );
}

// KlineNode 独特的日志消息
pub mod kline_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};
    use star_river_core::market::Exchange;

    log_message!(
        StartRegisterExchangeMsg,
        params: (
            node_name: String,
            exchange: Exchange,
        ),
        en: "[{node_name}] start to register exchange [{exchange}]",
        zh: "[{node_name}] 开始注册交易所: {exchange}"
    );

    log_message!(
        RegisterExchangeSuccessMsg,
        params: (
            node_name: String,
            exchange: Exchange,
        ),
        en: "[{node_name}] exchange [{exchange}] register successful",
        zh: "[{node_name}] 交易所注册成功"
    );

    log_message!(
        RegisterExchangeFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] exchange registration failed: {error}",
        zh: "[{node_name}] 交易所注册失败: {error}"
    );

    log_message!(
        StartLoadKlineDataMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to load kline data from exchange",
        zh: "[{node_name}] 开始从交易所加载K线数据"
    );

    log_message!(
        LoadKlineDataSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] kline data loading successful",
        zh: "[{node_name}] K线数据加载成功"
    );

    log_message!(
        LoadKlineDataFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] kline data loading failed: {error}",
        zh: "[{node_name}] K线数据加载失败: {error}"
    );

    log_message!(
        ProcessKlineSignalMsg,
        params: (
            node_name: String,
            signal_index: i32,
            play_index: i32
        ),
        en: "[{node_name}] received kline play signal, signal index: {signal_index}, node index: {play_index}",
        zh: "[{node_name}] 接收到K线播放信号，信号索引: {signal_index}, 节点索引: {play_index}"
    );

    log_message!(
        KlineIndexMismatchMsg,
        params: (
            node_name: String,
            cache_index: i32,
            signal_index: i32
        ),
        en: "[{node_name}] kline cache index mismatch - cache index: {cache_index}, signal index: {signal_index}",
        zh: "[{node_name}] K线缓存索引不匹配 - 缓存索引: {cache_index}, 信号索引: {signal_index}"
    );

    log_message!(
        SendKlineEventSuccessMsg,
        params: (
            node_name: String,
            symbol: String
        ),
        en: "[{node_name}] kline event sent successfully - Symbol: {symbol}",
        zh: "[{node_name}] K线事件发送成功 - 交易对: {symbol}"
    );

    log_message!(
        SendKlineEventFailedMsg,
        params: (
            node_name: String,
            symbol: String,
            error: String
        ),
        en: "[{node_name}] kline event sending failed - Symbol: {symbol}, Error: {error}",
        zh: "[{node_name}] K线事件发送失败 - 交易对: {symbol}, 错误: {error}"
    );
}

// IfElseNode 独特的日志消息
pub mod if_else_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    log_message!(
        ListenStrategySignalMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to listen strategy signal",
        zh: "[{node_name}] 开始监听策略信号"
    );

    log_message!(
        InitReceivedDataMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] initializing received data flags",
        zh: "[{node_name}] 初始化接收数据标记"
    );

    log_message!(
        InitReceivedDataSuccessMsg,
        params: (
            node_name: String,
            case_count: usize
        ),
        en: "[{node_name}] received data flags initialization successful, initialized {case_count} cases",
        zh: "[{node_name}] 接收数据标记初始化成功，已初始化 {case_count} 个条件"
    );

    log_message!(
        StartConditionEvaluationMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting condition evaluation process",
        zh: "[{node_name}] 开始条件评估进程"
    );

    log_message!(
        ConditionMatchedMsg,
        params: (
            node_name: String,
            case_index: i32,
        ),
        en: "[{node_name}] Case [{case_index}] matched",
        zh: "[{node_name}] 分支 [{case_index}] 已匹配"
    );
}

// PositionManagementNode 特有的日志消息
pub mod position_management_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    log_message!(
        ExecutePositionOperationMsg,
        params: (
            node_id: String,
            node_name: String,
            operation_id: i32,
            operation_type: String
        ),
        en: "PositionManagement Node [{node_name}] executing position operation - ID: {operation_id}, Type: {operation_type}",
        zh: "{node_name}  执行仓位操作 - ID: {operation_id}, 类型: {operation_type}"
    );

    log_message!(
        PositionOperationSuccessMsg,
        params: (
            node_id: String,
            node_name: String,
            operation_id: i32,
            operation_type: String
        ),
        en: "PositionManagement Node [{node_name}] position operation successful - ID: {operation_id}, Type: {operation_type}",
        zh: "{node_name}  仓位操作成功 - ID: {operation_id}, 类型: {operation_type}"
    );

    log_message!(
        PositionOperationFailedMsg,
        params: (
            node_id: String,
            node_name: String,
            operation_id: i32,
            operation_type: String,
            error: String
        ),
        en: "PositionManagement Node [{node_name}] position operation failed - ID: {operation_id}, Type: {operation_type}, Error: {error}",
        zh: "{node_name}  仓位操作失败 - ID: {operation_id}, 类型: {operation_type}, 错误: {error}"
    );

    log_message!(
        GetCurrentPositionMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String
        ),
        en: "PositionManagement Node [{node_name}] getting current position for symbol: {symbol}",
        zh: "{node_name}  获取交易对当前仓位: {symbol}"
    );

    log_message!(
        CurrentPositionMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            size: f64,
            side: String
        ),
        en: "PositionManagement Node [{node_name}] current position - Symbol: {symbol}, Size: {size}, Side: {side}",
        zh: "{node_name}  当前仓位 - 交易对: {symbol}, 数量: {size}, 方向: {side}"
    );

    log_message!(
        ClosePositionMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            quantity: f64
        ),
        en: "PositionManagement Node [{node_name}] closing position - Symbol: {symbol}, Quantity: {quantity}",
        zh: "{node_name}  平仓 - 交易对: {symbol}, 数量: {quantity}"
    );

    log_message!(
        ClosePositionSuccessMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            closed_quantity: f64
        ),
        en: "PositionManagement Node [{node_name}] position closed successfully - Symbol: {symbol}, Closed Quantity: {closed_quantity}",
        zh: "{node_name}  平仓成功 - 交易对: {symbol}, 平仓数量: {closed_quantity}"
    );

    log_message!(
        ClosePositionFailedMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            error: String
        ),
        en: "PositionManagement Node [{node_name}] position closing failed - Symbol: {symbol}, Error: {error}",
        zh: "{node_name}  平仓失败 - 交易对: {symbol}, 错误: {error}"
    );

    log_message!(
        AdjustPositionSizeMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            old_size: f64,
            new_size: f64
        ),
        en: "PositionManagement Node [{node_name}] adjusting position size - Symbol: {symbol}, From: {old_size}, To: {new_size}",
        zh: "{node_name}  调整仓位大小 - 交易对: {symbol}, 从: {old_size}, 到: {new_size}"
    );

    log_message!(
        CalculatePositionPnLMsg,
        params: (
            node_id: String,
            node_name: String,
            symbol: String,
            unrealized_pnl: f64,
            realized_pnl: f64
        ),
        en: "PositionManagement Node [{node_name}] position P&L calculated - Symbol: {symbol}, Unrealized: {unrealized_pnl}, Realized: {realized_pnl}",
        zh: "{node_name}  计算仓位盈亏 - 交易对: {symbol}, 浮动盈亏: {unrealized_pnl}, 已实现盈亏: {realized_pnl}"
    );

    log_message!(
        HandleVirtualTradingSystemEventMsg,
        params: (
            node_id: String,
            node_name: String,
            event_type: String
        ),
        en: "PositionManagement Node [{node_name}] handling virtual trading system event - Type: {event_type}",
        zh: "{node_name}  处理虚拟交易系统事件 - 类型: {event_type}"
    );
}

// VariableNode 特有的日志消息
pub mod variable_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};

    log_message!(
        GetVariableMsg,
        params: (
            node_name: String,
            variable_name: String,
            variable_type: String
        ),
        en: "[{node_name}] getting variable - Name: {variable_name}, Type: {variable_type}",
        zh: "[{node_name}] 获取变量 - 名称: {variable_name}, 类型: {variable_type}"
    );

    log_message!(
        GetVariableSuccessMsg,
        params: (
            node_id: String,
            node_name: String,
            variable_name: String,
            value: String
        ),
        en: "[{node_name}] variable retrieved successfully - Name: {variable_name}, Value: {value}",
        zh: "[{node_name}] 变量获取成功 - 名称: {variable_name}, 值: {value}"
    );

    log_message!(
        GetVariableFailedMsg,
        params: (
            node_name: String,
            variable_name: String,
            error: String
        ),
        en: "[{node_name}] variable retrieval failed - Name: {variable_name}, Error: {error}",
        zh: "[{node_name}] 变量获取失败 - 名称: {variable_name}, 错误: {error}"
    );

    log_message!(
        SetVariableMsg,
        params: (
            node_name: String,
            variable_name: String,
            value: String
        ),
        en: "[{node_name}] setting variable - Name: {variable_name}, Value: {value}",
        zh: "[{node_name}] 设置变量 - 名称: {variable_name}, 值: {value}"
    );

    log_message!(
        SetVariableSuccessMsg,
        params: (
            node_name: String,
            variable_name: String,
            value: String
        ),
        en: "[{node_name}] variable set successfully - Name: {variable_name}, Value: {value}",
        zh: "[{node_name}] 变量设置成功 - 名称: {variable_name}, 值: {value}"
    );

    log_message!(
        SetVariableFailedMsg,
        params: (
            node_name: String,
            variable_name: String,
            error: String
        ),
        en: "[{node_name}] variable setting failed - Name: {variable_name}, Error: {error}",
        zh: "[{node_name}] 变量设置失败 - 名称: {variable_name}, 错误: {error}"
    );

    log_message!(
        ProcessVariableConfigMsg,
        params: (
            node_name: String,
            config_id: i32,
            variable_count: usize
        ),
        en: "[{node_name}] processing variable config - Config ID: {config_id}, Variable Count: {variable_count}",
        zh: "[{node_name}] 处理变量配置 - 配置ID: {config_id}, 变量数量: {variable_count}"
    );

    log_message!(
        SendVariableValueMsg,
        params: (
            node_name: String,
            variable_name: String,
            output_handle_id: String,
            value: String
        ),
        en: "[{node_name}] sending variable value - Name: {variable_name}, Output: {output_handle_id}, Value: {value}",
        zh: "[{node_name}] 发送变量值 - 名称: {variable_name}, 输出: {output_handle_id}, 值: {value}"
    );

    log_message!(
        HandleNodeEventMsg,
        params: (
            node_name: String,
            from_node_id: String,
            event_type: String
        ),
        en: "[{node_name}] handling node event - From: {from_node_id}, Type: {event_type}",
        zh: "[{node_name}] 处理节点事件 - 来源: {from_node_id}, 类型: {event_type}"
    );

    log_message!(
        VariableStorageMsg,
        params: (
            node_name: String,
            operation: String,
            variable_name: String
        ),
        en: "[{node_name}] variable storage operation - Operation: {operation}, Variable: {variable_name}",
        zh: "[{node_name}] 变量存储操作 - 操作: {operation}, 变量: {variable_name}"
    );

    log_message!(
        ValidateVariableValueMsg,
        params: (
            node_name: String,
            variable_name: String,
            value: String,
            is_valid: bool
        ),
        en: "[{node_name}] variable value validation - Name: {variable_name}, Value: {value}, Valid: {is_valid}",
        zh: "[{node_name}] 变量值验证 - 名称: {variable_name}, 值: {value}, 有效: {is_valid}"
    );

    log_message!(
        RegisterVariableRetrievalTaskMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] registering variable retrieval task",
        zh: "[{node_name}] 注册变量获取任务"
    );

    log_message!(
        RegisterVariableRetrievalTaskSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] variable retrieval task registration successful",
        zh: "[{node_name}] 变量获取任务注册成功"
    );
}

// FuturesOrderNode 特有的日志消息
pub mod futures_order_node_log_message {
    use star_river_core::strategy::log_message::*;
    use star_river_core::log_message;
    use serde::{Deserialize, Serialize};
    use star_river_core::custom_type::OrderId;

    log_message!(
        ProcessingOrderMsg,
        params: (
            order_config_id: i32
        ),
        en: "processing order - Config ID: {order_config_id}, can't create a new order",
        zh: "正在处理订单 - 配置ID: {order_config_id}, 无法创建新订单"
    );

    log_message!(
        MonitorUnfilledOrderMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to monitor unfilled orders",
        zh: "{node_name}  开始监控未成交订单"
    );

    log_message!(
        OrderCreatedMsg,
        params: (
            order_id: OrderId,
            order_config_id: i32,
            price: f64,
            side: String
        ),
        en: "order created successfully - Order ID: {order_id}, Config ID: {order_config_id} price: {price} side: {side}",
        zh: "[{node_name}] 订单创建成功 - 订单ID: {order_id}, 配置ID: {order_config_id} 价格: {price} 方向: {side}"
    );

    log_message!(
        OrderPlacedMsg,
        params: (
            node_name: String,
            order_id: OrderId,
            order_config_id: i32
        ),
        en: "[{node_name}] order placed successfully - Order ID: {order_id}, Config ID: {order_config_id}",
        zh: "[{node_name}] 订单下单成功 - 订单ID: {order_id}, 配置ID: {order_config_id}"
    );

    log_message!(
        OrderPartialFilledMsg,
        params: (
            node_name: String,
            order_id: OrderId,
            filled_quantity: f64,
            remaining_quantity: f64
        ),
        en: "[{node_name}] order partially filled - Order ID: {order_id}, Filled: {filled_quantity}, Remaining: {remaining_quantity}",
        zh: "[{node_name}] 订单部分成交 - 订单ID: {order_id}, 已成交: {filled_quantity}, 剩余: {remaining_quantity}"
    );

    log_message!(
        OrderFilledMsg,
        params: (
            node_name: String,
            order_id: OrderId,
            filled_quantity: f64,
            filled_price: f64
        ),
        en: "[{node_name}] order completely filled - Order ID: {order_id}, Quantity: {filled_quantity}, Price: {filled_price}",
        zh: "[{node_name}] 订单完全成交 - 订单ID: {order_id}, 数量: {filled_quantity}, 价格: {filled_price}"
    );

    log_message!(
        OrderCanceledMsg,
        params: (
            node_name: String,
            order_id: OrderId,
        ),
        en: "[{node_name}] order canceled - Order ID: {order_id}",
        zh: "[{node_name}] 订单已取消 - 订单ID: {order_id}"
    );

    log_message!(
        OrderExpiredMsg,
        params: (
            node_name: String,
            order_id: OrderId
        ),
        en: "[{node_name}] order expired - Order ID: {order_id}",
        zh: "[{node_name}] 订单已过期 - 订单ID: {order_id}"
    );

    log_message!(
        OrderRejectedMsg,
        params: (
            node_name: String,
            order_id: String,
            reason: String
        ),
        en: "[{node_name}] order rejected - Order ID: {order_id}, Reason: {reason}",
        zh: "[{node_name}] 订单被拒绝 - 订单ID: {order_id}, 原因: {reason}"
    );

    log_message!(
        OrderErrorMsg,
        params: (
            node_name: String,
            order_id: String,
            error: String
        ),
        en: "[{node_name}] order error - Order ID: {order_id}, Error: {error}",
        zh: "[{node_name}] 订单错误 - 订单ID: {order_id}, 错误: {error}"
    );

    log_message!(
        HandleVirtualTradingSystemEventMsg,
        params: (
            node_name: String,
            event_type: String
        ),
        en: "FuturesOrder Node [{node_name}] handling virtual trading system event - Type: {event_type}",
        zh: "{node_name}  处理虚拟交易系统事件 - 类型: {event_type}"
    );

    log_message!(
        HandleNodeEventForSpecificOrderMsg,
        params: (
            node_name: String,
            input_handle_id: String,
            order_config_id: i32
        ),
        en: "[{node_name}] handling node event for specific order - Input: {input_handle_id}, Order Config: {order_config_id}",
        zh: "[{node_name}] 处理特定订单的节点事件 - 输入: {input_handle_id}, 订单配置: {order_config_id}"
    );

    log_message!(
        TakeProfitTriggeredMsg,
        params: (
            node_name: String,
            order_id: String,
            tp_price: f64,
            current_price: f64
        ),
        en: "[{node_name}] take profit triggered - Order ID: {order_id}, TP Price: {tp_price}, Current Price: {current_price}",
        zh: "[{node_name}] 止盈触发 - 订单ID: {order_id}, 止盈价: {tp_price}, 当前价: {current_price}"
    );

    log_message!(
        StopLossTriggeredMsg,
        params: (
            node_name: String,
            order_id: String,
            sl_price: f64,
            current_price: f64
        ),
        en: "[{node_name}] stop loss triggered - Order ID: {order_id}, SL Price: {sl_price}, Current Price: {current_price}",
        zh: "[{node_name}] 止损触发 - 订单ID: {order_id}, 止损价: {sl_price}, 当前价: {current_price}"
    );

    log_message!(
        NodeEventReceiverTerminatedMsg,
        params: (
            node_name: String,
            input_handle_id: String
        ),
        en: "[{node_name}] node event receiver terminated - Input: {input_handle_id}",
        zh: "[{node_name}] 节点事件接收器已终止 - 输入: {input_handle_id}"
    );

    log_message!(
        GetSymbolInfoMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] starting to get symbol info",
        zh: "[{node_name}] 开始获取交易对信息"
    );

    log_message!(
        GetSymbolInfoSuccessMsg,
        params: (
            node_name: String
        ),
        en: "[{node_name}] symbol info retrieved successfully",
        zh: "[{node_name}] 交易对信息获取成功"
    );
    
    log_message!(
        GetSymbolInfoFailedMsg,
        params: (
            node_name: String,
            error: String
        ),
        en: "[{node_name}] symbol info retrieval failed - Error: {error}",
        zh: "[{node_name}] 交易对信息获取失败 - 错误: {error}"
    );
}

// OrderNode 特有的日志消息 (保留以便未来扩展)
pub mod order_node_log_message {
    // 暂时空置，等待OrderNode实现
}
