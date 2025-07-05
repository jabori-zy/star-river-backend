use types::strategy::Strategy;
use crate::strategy_config::Model as StrategyConfigModel;
use types::strategy::TradeMode;
use std::str::FromStr;

impl From<StrategyConfigModel> for Strategy {
    fn from(config: StrategyConfigModel) -> Self {
        Strategy {
            id: config.id,
            name: config.name,
            description: config.description,
            status: config.status,
            is_deleted: config.is_deleted,
            trade_mode: TradeMode::from_str(config.trade_mode.as_str()).unwrap(),
            config: config.config,
            nodes: config.nodes,
            edges: config.edges,
            live_chart_config: config.live_chart_config,
            backtest_chart_config: config.backtest_chart_config,
            created_time: config.created_time,
            updated_time: config.updated_time,
        }
    }
}
