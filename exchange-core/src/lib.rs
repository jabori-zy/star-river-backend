pub mod exchange;
pub mod exchange_trait;
pub mod state_machine;
pub mod error;

pub use exchange::ExchangeBase;
pub use exchange_trait::{
    DataProcessor, ExchangeAccountExt, ExchangeLifecycle, ExchangeMarketDataExt, ExchangeMetadata,
    ExchangePositionExt, HttpClient, MetadataAccessor, ProcessorAccessor, WebSocketClient,
};
pub use state_machine::{
    ExchangeAction, ExchangeRunState, ExchangeStateMachine, ExchangeStateTransTrigger, Metadata,
    StateChangeActions,
};

// Re-export from star-river-core for convenience
pub use star_river_core::{
    account::OriginalAccountInfo,
    market::{Kline, KlineInterval},
    position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position, PositionNumber},
    strategy::TimeRange,
};
