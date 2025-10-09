use super::{
    ExchangeAccountExt,
    Binance,
    ExchangeClientError,
    async_trait,
};
use star_river_core::account::OriginalAccountInfo;

#[async_trait]
impl ExchangeAccountExt for Binance {
    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError> {
        todo!()
    }
}
