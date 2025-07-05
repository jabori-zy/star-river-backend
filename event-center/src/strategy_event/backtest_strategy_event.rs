use serde::{Serialize, Deserialize};
use strum::Display;
use types::strategy::node_event::backtest_node_event::kline_event::KlineUpdateEvent;
use types::strategy::node_event::IndicatorUpdateEvent;
use crate::{StrategyEvent, Event};



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum BacktestStrategyEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineUpdateEvent), // 回测K线更新事件

    #[strum(serialize = "indicator-update")]
    #[serde(rename = "indicator-update")]
    IndicatorUpdate(IndicatorUpdateEvent), // 回测指标更新事件
}

impl From<BacktestStrategyEvent> for Event {
    fn from(event: BacktestStrategyEvent) -> Self {
        StrategyEvent::BacktestStrategy(event).into()
    }
}