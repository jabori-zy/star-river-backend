pub mod error;
pub mod exchange;
pub mod exchange_trait;
pub mod state_machine;

pub use exchange::ExchangeBase;
pub use exchange_trait::{
    DataProcessor, ExchangeAccountExt, ExchangeLifecycle, ExchangeMarketDataExt, ExchangeMetadata, ExchangePositionExt, HttpClient,
    MetadataAccessor, ProcessorAccessor, WebSocketClient,
};
// Re-export from star-river-core for convenience
pub use star_river_core::{
    account::OriginalAccountInfo,
    exchange::Exchange as ExchangeType,
    kline::{Kline, KlineInterval},
    position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position, PositionNumber},
};
pub use state_machine::{ExchangeAction, ExchangeRunState, ExchangeStateMachine, ExchangeStateTransTrigger, Metadata, StateChangeActions};
