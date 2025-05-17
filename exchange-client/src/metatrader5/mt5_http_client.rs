use crate::metatrader5::url::Mt5HttpUrl;
use crate::metatrader5::mt5_types::Mt5KlineInterval;
use serde::Serialize;

use super::mt5_types::Mt5GetPositionNumberParams;
use super::mt5_types::Mt5CreateOrderParams;
use thiserror::Error;
use tracing::instrument;



#[derive(Error, Debug)]
pub enum Mt5HttpClientError {
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("InitializeTerminalError: {0}")]
    InitializeTerminalError(String),
    #[error("GetTerminalInfoError: {0}")]
    GetTerminalInfoError(String),
    #[error("GetKlineSeriesError: {0}")]
    GetKlineSeriesError(String),
    #[error("CreateOrderError: {0}")]
    CreateOrderError(String),
    #[error("GetOrderError: {0}")]
    GetOrderError(String),
    #[error("GetPositionError: {0}")]
    GetPositionError(String),
    #[error("GetDealError: {0}")]
    GetDealError(String),
    #[error("GetPositionNumberError: {0}")]
    GetPositionNumberError(String),
    #[error("GetAccountInfoError: {0}")]
    GetAccountInfoError(String),
    #[error("PingError: {0}")]
    PingError(String),
}







#[derive(Debug)]
pub struct Mt5HttpClient {
    port: u16,
    client: reqwest::Client,
}


impl Mt5HttpClient {
    pub fn new(port: u16) -> Self {
        Self {
            port: port,
            client: reqwest::Client::new(),
        }
    }

    fn get_url(&self, mt5_http_url: Mt5HttpUrl) -> String {
        format!("{}:{}{}", Mt5HttpUrl::BaseUrl, self.port, mt5_http_url)
    }

    #[instrument(skip(self))]
    pub async fn ping(&self) -> Result<(), Mt5HttpClientError> {
        let url = self.get_url(Mt5HttpUrl::Ping);
        tracing::debug!(url = %url, "Pinging MT5 server");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        if response.status().is_success() {
            tracing::debug!("ping MT5 server success");
            Ok(())
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, "Failed to ping MT5 server");
            Err(Mt5HttpClientError::PingError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    // 初始化MT5客户端
    #[instrument(skip(self, password), fields(login = %login, server = %server, terminal_path = %terminal_path))]
    pub async fn initialize_terminal(
        &self,
        login: i64,
        password: &str,
        server: &str,
        terminal_path: &str
    ) -> Result<(), Mt5HttpClientError> {
        tracing::info!("Start to initialize MT5 terminal");
        let url = self.get_url(Mt5HttpUrl::InitializeTerminal);
        tracing::debug!(url = %url, "Initializing MT5 terminal");
        #[derive(Debug, Serialize)]
        struct InitializeTerminalRequest {
            login: i64,
            password: String,
            server: String,
            terminal_path: String,
        }
        let request = InitializeTerminalRequest {
            login: login,
            password: password.to_string(),
            server: server.to_string(),
            terminal_path: terminal_path.to_string(),
        };
        let response = self.client.post(&url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则初始化成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.map_err(|e| Mt5HttpClientError::HttpError(e))?;
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                if code == 0 {
                    tracing::info!("MT5 terminal initialized successfully");
                    return Ok(())
                } else {
                    // 获取错误信息
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the init terminal response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to initialize MT5 terminal");
                    Err(Mt5HttpClientError::InitializeTerminalError(error_message))
                }
            } else {
                let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                tracing::error!(error = %error_message, "Failed to initialize MT5 terminal - invalid response format");
                Err(Mt5HttpClientError::InitializeTerminalError(error_message))
            }
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
            .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            // 如果是其他状态码，则返回错误
            tracing::error!(status = %status_code, error = %error_text, "Failed to initialize MT5 terminal - HTTP error");
            Err(Mt5HttpClientError::InitializeTerminalError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }


    #[instrument(skip(self))]
    pub async fn get_terminal_info(&self) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = self.get_url(Mt5HttpUrl::GetTerminalInfo);
        tracing::debug!(url = %url, "Getting terminal info");
        let response = self.client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取终端信息
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.map_err(|e| Mt5HttpClientError::HttpError(e))?;
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!("Successfully got terminal info");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get terminal info response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to get terminal info");
                    Err(Mt5HttpClientError::GetTerminalInfoError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to get terminal info");
                Err(Mt5HttpClientError::GetTerminalInfoError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
            .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            // 如果是其他状态码，则返回错误
            tracing::error!(status = %status_code, error = %error_text, "Failed to get terminal info - HTTP error");
            Err(Mt5HttpClientError::GetTerminalInfoError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    // 获取K线系列
    #[instrument(skip(self))]
    pub async fn get_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, limit: u32) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?symbol={}&interval={}&limit={}", self.get_url(Mt5HttpUrl::GetKlineSeries), symbol, interval, limit);
        tracing::debug!(url = %url, symbol = %symbol, interval = %interval, limit = %limit, "Getting kline series");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取K线数据成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(symbol = %symbol, "Successfully got kline series");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get kline series response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, symbol = %symbol, "Failed to get kline series");
                    Err(Mt5HttpClientError::GetKlineSeriesError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, symbol = %symbol, "Failed to get kline series");
                Err(Mt5HttpClientError::GetKlineSeriesError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, symbol = %symbol, "Failed to get kline series - HTTP error");
            Err(Mt5HttpClientError::GetKlineSeriesError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn create_order(&self, params: Mt5CreateOrderParams) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = self.get_url(Mt5HttpUrl::CreateOrder);
        tracing::debug!(url = %url, params = ?params, "Creating order");

        let response = self.client.post(&url)
        .json(&params)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则创建订单成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::info!("Successfully created order");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the create order response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to create order");
                    Err(Mt5HttpClientError::CreateOrderError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to create order");
                Err(Mt5HttpClientError::CreateOrderError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, "Failed to create order - HTTP error");
            Err(Mt5HttpClientError::CreateOrderError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    // 获取订单
    #[instrument(skip(self))]
    pub async fn get_order(&self, order_id: &i64) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?order_id={}", self.get_url(Mt5HttpUrl::GetOrder), order_id);
        tracing::debug!(url = %url, order_id = %order_id, "Getting order");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取订单成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(order_id = %order_id, "Successfully got order");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get order response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, order_id = %order_id, "Failed to get order");
                    Err(Mt5HttpClientError::GetOrderError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, order_id = %order_id, "Failed to get order");
                Err(Mt5HttpClientError::GetOrderError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, order_id = %order_id, "Failed to get order - HTTP error");
            Err(Mt5HttpClientError::GetOrderError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_position(&self, position_id: &i64) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?position_id={}", self.get_url(Mt5HttpUrl::GetPosition), position_id);
        tracing::debug!(url = %url, position_id = %position_id, "Getting position");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取仓位成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(position_id = %position_id, "Successfully got position");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get position response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, position_id = %position_id, "Failed to get position");
                    Err(Mt5HttpClientError::GetPositionError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, position_id = %position_id, "Failed to get position");
                Err(Mt5HttpClientError::GetPositionError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, position_id = %position_id, "Failed to get position - HTTP error");
            Err(Mt5HttpClientError::GetPositionError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    // 获取成交明细
    #[instrument(skip(self))]
    pub async fn get_deal_by_position_id(&self, position_id: &i64) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?position_id={}", self.get_url(Mt5HttpUrl::GetDeal), position_id);
        tracing::debug!(url = %url, position_id = %position_id, "Getting deal by position id");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(position_id = %position_id, "Successfully got deal by position id");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get deal response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, position_id = %position_id, "Failed to get deal by position id");
                    Err(Mt5HttpClientError::GetDealError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, position_id = %position_id, "Failed to get deal by position id");
                Err(Mt5HttpClientError::GetDealError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, position_id = %position_id, "Failed to get deal by position id - HTTP error");
            Err(Mt5HttpClientError::GetDealError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_deal_by_deal_id(&self, deal_id: &i64) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?deal_id={}", self.get_url(Mt5HttpUrl::GetDeal), deal_id);
        tracing::debug!(url = %url, deal_id = %deal_id, "Getting deal by deal id");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(deal_id = %deal_id, "Successfully got deal by deal id");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get deal response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, deal_id = %deal_id, "Failed to get deal by deal id");
                    Err(Mt5HttpClientError::GetDealError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, deal_id = %deal_id, "Failed to get deal by deal id");
                Err(Mt5HttpClientError::GetDealError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, deal_id = %deal_id, "Failed to get deal by deal id - HTTP error");
            Err(Mt5HttpClientError::GetDealError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_deals_by_order_id(&self, order_id: &i64) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = format!("{}?order_id={}", self.get_url(Mt5HttpUrl::GetDeal), order_id);
        tracing::debug!(url = %url, order_id = %order_id, "Getting deals by order id");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(order_id = %order_id, "Successfully got deals by order id");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get deals response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, order_id = %order_id, "Failed to get deals by order id");
                    Err(Mt5HttpClientError::GetDealError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, order_id = %order_id, "Failed to get deals by order id");
                Err(Mt5HttpClientError::GetDealError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, order_id = %order_id, "Failed to get deals by order id - HTTP error");
            Err(Mt5HttpClientError::GetDealError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_position_number(&self, position_number_request: Mt5GetPositionNumberParams) -> Result<serde_json::Value, Mt5HttpClientError> {
        let symbol = &position_number_request.symbol;
        let position_side = position_number_request.position_side;
        
        let url = format!("{}?symbol={}{}",
            self.get_url(Mt5HttpUrl::GetPositionNumber),
            symbol,
            position_side.clone().map_or(String::new(), |side| format!("&position_side={}", side))
        );
        
        tracing::debug!(url = %url, symbol = %symbol, position_side = ?position_side, "Getting position number");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取仓位数量成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(symbol = %symbol, position_side = ?position_side, "Successfully got position number");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get position number response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, symbol = %symbol, position_side = ?position_side, "Failed to get position number");
                    Err(Mt5HttpClientError::GetPositionNumberError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, symbol = %symbol, position_side = ?position_side, "Failed to get position number");
                Err(Mt5HttpClientError::GetPositionNumberError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, symbol = %symbol, position_side = ?position_side, "Failed to get position number - HTTP error");
            Err(Mt5HttpClientError::GetPositionNumberError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_account_info(&self) -> Result<serde_json::Value, Mt5HttpClientError> {
        let url = self.get_url(Mt5HttpUrl::GetAccountInfo);
        tracing::debug!(url = %url, "Getting account info");
        
        let response = self.client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| Mt5HttpClientError::HttpError(e))?;

        // 如果为200，则获取账户信息成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!("Successfully got account info");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&format!("unknown error, the get account info response code is {}", code))
                    .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to get account info");
                    Err(Mt5HttpClientError::GetAccountInfoError(error_message))
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to get account info");
                Err(Mt5HttpClientError::GetAccountInfoError(error_message))
            }
        } 
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await
                .map_err(|e| Mt5HttpClientError::HttpError(e))?;
            
            tracing::error!(status = %status_code, error = %error_text, "Failed to get account info - HTTP error");
            Err(Mt5HttpClientError::GetAccountInfoError(format!("status code: {}, error text: {}", status_code, error_text)))
        }
    }

}




