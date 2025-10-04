use super::{
    ExchangeOrderExt,
    MetaTrader5,
    ExchangeClientError,
    Mt5CreateOrderParams,
    HttpClientNotCreatedSnafu,
    OtherSnafu,
    RetcodeSnafu,
    OrderIdSnafu,
};
use async_trait::async_trait;
use star_river_core::{
    order::{CreateOrderParams, GetTransactionDetailParams, OriginalOrder}, 
    transaction::OriginalTransaction,
    order::Order,
};
use snafu::OptionExt;


#[async_trait]
impl ExchangeOrderExt for MetaTrader5 {
    async fn create_order(&self, params: CreateOrderParams) -> Result<Box<dyn OriginalOrder>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        let mt5_order_request = Mt5CreateOrderParams::from(params);

        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            // 创建订单
            let create_order_result = mt5_http_client.create_order(mt5_order_request).await?;

            // 获取返回码
            let retcode = create_order_result["data"]["retcode"].as_i64().context(RetcodeSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            })?;

            if retcode != 10009 {
                return RetcodeSnafu {
                    terminal_id: self.terminal_id,
                    port: self.server_port,
                }
                .fail()?;
            }

            // 获取订单ID
            let order_id = create_order_result["data"]["order_id"].as_i64().context(OrderIdSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            })?;

            // 获取订单详情
            let order_info = mt5_http_client.get_order(&order_id).await?;

            // 处理订单数据
            let data_processor = self.data_processor.lock().await;
            let order = data_processor.process_order(order_info).await?;
            Ok(order)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn update_order(&self, order: Order) -> Result<Order, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let order_info = mt5_http_client.get_order(&order.exchange_order_id).await?;

            let data_processor = self.data_processor.lock().await;
            let updated_order = data_processor.update_order(order_info, order).await?;
            Ok(updated_order)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
    
    
    async fn get_transaction_detail(
        &self,
        params: GetTransactionDetailParams,
    ) -> Result<Box<dyn OriginalTransaction>, ExchangeClientError> {
        let mt5_http_client: tokio::sync::MutexGuard<'_, Option<crate::metatrader5::mt5_http_client::Mt5HttpClient>> = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let data_processor = self.data_processor.lock().await;

            if let Some(transaction_id) = params.transaction_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_deal_id(&transaction_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else if let Some(position_id) = params.position_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_position_id(&position_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else if let Some(order_id) = params.order_id {
                let transaction_detail_info = mt5_http_client.get_deals_by_order_id(&order_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else {
                return OtherSnafu {
                    message: "transaction_id, position_id, order_id cannot be None".to_string(),
                }
                .fail()?;
            }
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
}