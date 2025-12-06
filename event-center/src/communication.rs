use derive_more::From;
use event_center_core::{CommandTarget, Target};
use star_river_core::engine::EngineName;
use star_river_event::communication::{ExchangeEngineCommand, IndicatorEngineCommand, MarketEngineCommand};
use strum::{Display, EnumIter, IntoEnumIterator};

// Define engine names (example)
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
    IndicatorEngine(IndicatorEngineCommand), // Indicator engine command
    MarketEngine(MarketEngineCommand),       // Market engine command
    ExchangeEngine(ExchangeEngineCommand),   // Exchange engine command
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
