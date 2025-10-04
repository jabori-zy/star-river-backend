use super::{
    ExchangePositionExt,
    MetaTrader5,
    ExchangeClientError,
    HttpClientNotCreatedSnafu,
    OtherSnafu,
    Mt5GetPositionNumberParams,
};
use async_trait::async_trait;
use star_river_core::position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position, PositionNumber};


#[async_trait]
impl ExchangePositionExt for MetaTrader5 {
    async fn get_position(&self, params: GetPositionParam) -> Result<Box<dyn OriginalPosition>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let position_info = mt5_http_client.get_position(&params.position_id).await?;
            let position_list = position_info["data"].clone();
            // 如果仓位列表为空，则说明仓位已平仓
            if position_list.as_array().expect("转换为array失败").len() == 0 {
                return OtherSnafu {
                    message: "仓位已平仓".to_string(),
                }
                .fail()?;
            }
            let data_processor = self.data_processor.lock().await;
            let position = data_processor.process_position(position_list[0].clone()).await?;
            Ok(position)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_latest_position(&self, position: &Position) -> Result<Position, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let original_position_json = mt5_http_client
                .get_position(&position.exchange_position_id)
                .await
                .expect("更新仓位失败");
            let position_list = original_position_json["data"].clone();
            // 如果仓位列表为空，则说明仓位已平仓
            if position_list.as_array().expect("转换为array失败").len() == 0 {
                return OtherSnafu {
                    message: "仓位已平仓".to_string(),
                }
                .fail()?;
            }
            let data_processor = self.data_processor.lock().await;
            let position = data_processor
                .process_latest_position(position_list[0].clone(), position)
                .await
                .expect("处理仓位失败");
            Ok(position)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_position_number(&self, position_number_request: GetPositionNumberParams) -> Result<PositionNumber, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let mt5_position_number_request = Mt5GetPositionNumberParams::from(position_number_request);
            let position_number_info = mt5_http_client
                .get_position_number(mt5_position_number_request)
                .await
                .expect("获取仓位数量失败");
            let mt5_data_processor = self.data_processor.lock().await;
            let position_number = mt5_data_processor
                .process_position_number(position_number_info)
                .await
                .expect("解析position_number数据失败");
            Ok(position_number)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
}