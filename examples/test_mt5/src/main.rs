
use event_center::command_event::market_engine_command::SubscribeKlineStreamParams;
use event_center::{Channel, EventCenter};
use database::DatabaseManager;
use sea_orm::prelude::Uuid;
use types::market::Exchange;
use std::time::Duration;
use tracing::{Event, Level};
use tracing_subscriber::EnvFilter;
use event_center::command_event::CommandEvent;
use engine::engine_manager::EngineManager;
use event_center::command_event::exchange_engine_command::RegisterExchangeParams;
use event_center::command_event::exchange_engine_command::ExchangeEngineCommand;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::order::{OrderType, OrderSide};
use event_center::command_event::order_engine_command::CreateOrderParams;
use event_center::command_event::order_engine_command::OrderEngineCommand;
use event_center::command_event::BaseCommandParams;
use event_center::command_event::exchange_engine_command::RegisterMt5ExchangeParams;
use event_center::command_event::market_engine_command::MarketEngineCommand;
use types::market::KlineInterval;
use exchange_client::metatrader5::MetaTrader5;


#[tokio::main]
async fn main() {

    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let event_center = EventCenter::new();
    let event_publisher = event_center.get_event_publisher();

    let mut mt5_client = MetaTrader5::new(
        1,
        123123,
        "123456".to_string(),
        "123456".to_string(),
        "123456".to_string(),
        event_publisher.clone()
    );

    mt5_client.start_mt5_server(true).await.unwrap();

    let mut mt5_client2 = MetaTrader5::new(
        2,
        123123,
        "123456".to_string(),
        "123456".to_string(),
        "123456".to_string(),
        event_publisher
    );

    mt5_client2.start_mt5_server(true).await.unwrap();
}



