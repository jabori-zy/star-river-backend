use star_river_event::communication::{IndicatorEngineCommand, MarketEngineCommand, ExchangeEngineCommand};
use derive_more::From;
use strum::{EnumIter, IntoEnumIterator, Display};
use event_center_core::CommandTarget;
use event_center_core::Target;
use star_river_core::engine::EngineName;





// 定义引擎名称（示例）
#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum CommandTargetEngine {
    ExchangeEngine,
    MarketEngine,
    IndicatorEngine,
    BacktestEngine,
}


impl CommandTarget for CommandTargetEngine {
    fn variants() -> Vec<Self> {
        CommandTargetEngine::iter().collect()
    }
}


impl From<EngineName> for CommandTargetEngine {
    fn from(engine_name: EngineName) -> Self {
        match engine_name {
            EngineName::ExchangeEngine => CommandTargetEngine::ExchangeEngine,
            EngineName::MarketEngine => CommandTargetEngine::MarketEngine,
            EngineName::IndicatorEngine => CommandTargetEngine::IndicatorEngine,
            EngineName::BacktestEngine => CommandTargetEngine::BacktestEngine,
        }
    }
}







#[derive(Debug, From)]
pub enum EngineCommand {
    IndicatorEngine(IndicatorEngineCommand), // 指标引擎命令
    MarketEngine(MarketEngineCommand),       // 市场引擎命令
    ExchangeEngine(ExchangeEngineCommand),   // 交易所引擎命令
}

impl Target for EngineCommand {
    type T = CommandTargetEngine;
    fn target(&self) -> &Self::T {
        match self {
            EngineCommand::IndicatorEngine(_) => &CommandTargetEngine::IndicatorEngine,
            EngineCommand::MarketEngine(_) => &CommandTargetEngine::MarketEngine,
            EngineCommand::ExchangeEngine(_) => &CommandTargetEngine::ExchangeEngine,
        }
    }
}
