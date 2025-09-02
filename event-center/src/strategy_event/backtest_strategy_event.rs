use serde::{Serialize, Deserialize};
use strum::Display;
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineUpdateEvent;
use types::strategy::node_event::IndicatorUpdateEvent;
use crate::{StrategyEvent, Event};
use types::strategy::node_event::backtest_node_event::futures_order_node_event::{
    FuturesOrderCreatedEvent, 
    FuturesOrderCanceledEvent, 
    FuturesOrderFilledEvent, 
    TakeProfitOrderCreatedEvent, 
    TakeProfitOrderFilledEvent, 
    TakeProfitOrderCanceledEvent, 
    StopLossOrderCreatedEvent, 
    StopLossOrderFilledEvent, 
    StopLossOrderCanceledEvent,
    TransactionCreatedEvent
};
use types::strategy::node_event::backtest_node_event::position_management_node_event::{PositionCreatedEvent, PositionUpdatedEvent, PositionClosedEvent};
use types::strategy_stats::event::StrategyStatsUpdatedEvent;
use types::strategy::node_event::NodeStartLogEvent;
use types::strategy::node_event::LogLevel;




#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum BacktestStrategyEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineUpdateEvent), // 回测K线更新事件

    #[strum(serialize = "indicator-update")]
    #[serde(rename = "indicator-update")]
    IndicatorUpdate(IndicatorUpdateEvent), // 回测指标更新事件

    #[strum(serialize = "futures-order-filled")]
    #[serde(rename = "futures-order-filled")]
    FuturesOrderFilled(FuturesOrderFilledEvent), // 期货订单成交事件

    #[strum(serialize = "futures-order-created")]
    #[serde(rename = "futures-order-created")]
    FuturesOrderCreated(FuturesOrderCreatedEvent), // 期货订单创建事件

    #[strum(serialize = "futures-order-canceled")]
    #[serde(rename = "futures-order-canceled")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent), // 期货订单取消事件

    #[strum(serialize = "take-profit-order-created")]
    #[serde(rename = "take-profit-order-created")]
    TakeProfitOrderCreated(TakeProfitOrderCreatedEvent), // 止盈订单创建事件

    #[strum(serialize = "take-profit-order-filled")]
    #[serde(rename = "take-profit-order-filled")]
    TakeProfitOrderFilled(TakeProfitOrderFilledEvent), // 止盈订单成交事件

    #[strum(serialize = "take-profit-order-canceled")]
    #[serde(rename = "take-profit-order-canceled")]
    TakeProfitOrderCanceled(TakeProfitOrderCanceledEvent), // 止盈订单取消事件

    #[strum(serialize = "stop-loss-order-created")]
    #[serde(rename = "stop-loss-order-created")]
    StopLossOrderCreated(StopLossOrderCreatedEvent), // 止损订单创建事件

    #[strum(serialize = "stop-loss-order-filled")]
    #[serde(rename = "stop-loss-order-filled")]
    StopLossOrderFilled(StopLossOrderFilledEvent), // 止损订单成交事件

    #[strum(serialize = "stop-loss-order-canceled")]
    #[serde(rename = "stop-loss-order-canceled")]
    StopLossOrderCanceled(StopLossOrderCanceledEvent), // 止损订单取消事件

    #[strum(serialize = "position-created")]
    #[serde(rename = "position-created")]
    PositionCreated(PositionCreatedEvent), // 仓位创建事件

    #[strum(serialize = "position-updated")]
    #[serde(rename = "position-updated")]
    PositionUpdated(PositionUpdatedEvent), // 仓位更新事件

    #[strum(serialize = "position-closed")]
    #[serde(rename = "position-closed")]
    PositionClosed(PositionClosedEvent), // 仓位关闭事件


    #[strum(serialize = "strategy-stats-updated")]
    #[serde(rename = "strategy-stats-updated")]
    StrategyStatsUpdated(StrategyStatsUpdatedEvent), // 策略统计更新事件

    #[strum(serialize = "transaction-created")]
    #[serde(rename = "transaction-created")]
    TransactionCreated(TransactionCreatedEvent), // 交易明细创建事件

    #[strum(serialize = "state-log")]
    #[serde(rename = "state-log")]
    NodeStartLog(NodeStartLogEvent), // 启动日志事件

    #[strum(serialize = "strategy-state-log")]
    #[serde(rename = "strategy-state-log")]
    StrategyStartLog(StrategyStartLogEvent), // 策略启动日志事件
}

impl From<BacktestStrategyEvent> for Event {
    fn from(event: BacktestStrategyEvent) -> Self {
        StrategyEvent::BacktestStrategy(event).into()
    }
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStartLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "strategyName")]
    pub strategy_name: String,

    #[serde(rename = "strategyState")]
    pub strategy_state: Option<String>,

    #[serde(rename = "strategyStateAction")]
    pub strategy_state_action: Option<String>,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "errorCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,


    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "detail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(rename = "duration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}
