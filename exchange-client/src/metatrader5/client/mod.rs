mod market_data;
mod stream;
mod symbol;
mod account;
mod position;
mod order;

use super::{
    MetaTrader5,
    Exchange,
    ExchangeClientCore,
    ExchangeStatus,
};
use async_trait::async_trait;
use std::any::Any;
use super::super::exchange_trait::*;
use star_river_core::{
    market::{KlineInterval, Kline},
    error::exchange_client_error::{
        ExchangeClientError,
        mt5_error::*,
    },
    account::OriginalAccountInfo,
};
use super::mt5_types::*;
use star_river_core::strategy::TimeRange;
use super::mt5_ws_client::Mt5WsClient;




#[async_trait]
impl ExchangeClientCore for MetaTrader5 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ExchangeClientCore> {
        Box::new(self.clone())
    }

    fn exchange_type(&self) -> Exchange {
        Exchange::Metatrader5(self.server.clone())
    }

    fn get_status(&self) -> ExchangeStatus {
        self.status.clone()
    }

    fn set_status(&mut self, status: ExchangeStatus) {
        self.status = status;
    }
}