// pub mod live_strategy;
pub mod backtest_strategy;

use event_center::communication::backtest_strategy::{StrategyCommand, BacktestStrategyCommand, StrategyCommandSender};
use star_river_core::custom_type::NodeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;


