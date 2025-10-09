use super::{
    ExchangeOrderExt,
    Binance,
    ExchangeClientError,
    async_trait,
};
use star_river_core::{
    order::{CreateOrderParams, GetTransactionDetailParams, OriginalOrder, Order},
    transaction::OriginalTransaction,
};

#[async_trait]
impl ExchangeOrderExt for Binance {
    async fn create_order(&self, _params: CreateOrderParams) -> Result<Box<dyn OriginalOrder>, ExchangeClientError> {
        todo!()
    }

    async fn update_order(&self, _order: Order) -> Result<Order, ExchangeClientError> {
        todo!()
    }

    async fn get_transaction_detail(
        &self,
        _params: GetTransactionDetailParams,
    ) -> Result<Box<dyn OriginalTransaction>, ExchangeClientError> {
        todo!()
    }
}
