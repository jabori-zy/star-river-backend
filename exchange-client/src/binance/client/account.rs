use super::{
    ExchangeAccountExt,
    BinanceExchange,
    ExchangeClientError,
    async_trait,
};
use star_river_core::account::OriginalAccountInfo;

#[async_trait]
impl ExchangeAccountExt for BinanceExchange {
    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError> {
        todo!()
    }
}
