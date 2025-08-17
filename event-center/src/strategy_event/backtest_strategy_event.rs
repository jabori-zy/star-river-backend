use serde::{Serialize, Deserialize};
use strum::Display;
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineUpdateEvent;
use types::strategy::node_event::IndicatorUpdateEvent;
use crate::{StrategyEvent, Event};
use types::order::virtual_order::VirtualOrder;
use types::strategy::node_event::backtest_node_event::futures_order_node_event::{FuturesOrderCreatedEvent, FuturesOrderCanceledEvent, FuturesOrderFilledEvent};
use types::strategy::node_event::backtest_node_event::position_management_node_event::{PositionCreatedEvent, PositionUpdatedEvent, PositionClosedEvent};
use types::strategy_stats::event::StrategyStatsUpdatedEvent;




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
}

impl From<BacktestStrategyEvent> for Event {
    fn from(event: BacktestStrategyEvent) -> Self {
        StrategyEvent::BacktestStrategy(event).into()
    }
}