use super::{
    ExchangeAccountExt,
    MetaTrader5,
    OriginalAccountInfo,
    ExchangeClientError,
    HttpClientNotCreatedSnafu,
};
use async_trait::async_trait;


#[async_trait]
impl ExchangeAccountExt for MetaTrader5 {
    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let account_info = mt5_http_client.get_account_info().await?;
            let data_processor = self.data_processor.lock().await;
            let account_info = data_processor.process_account_info(self.terminal_id, account_info).await?;
            Ok(account_info)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
}