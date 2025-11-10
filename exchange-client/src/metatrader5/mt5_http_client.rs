use exchange_core::exchange_trait::HttpClient;
use serde::Serialize;
use snafu::{OptionExt, ResultExt};
use star_river_core::system::TimeRange;
use tracing::instrument;

use super::{
    error::*,
    mt5_types::{Mt5CreateOrderParams, Mt5GetPositionNumberParams},
};
use crate::metatrader5::{mt5_types::Mt5KlineInterval, url::Mt5HttpUrl};

#[derive(Debug)]
pub struct Mt5HttpClient {
    terminal_id: i32,
    port: Option<u16>,
    client: reqwest::Client,
}

impl HttpClient for Mt5HttpClient {}

impl Mt5HttpClient {
    pub fn new(terminal_id: i32) -> Self {
        Self {
            terminal_id,
            port: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = Some(port);
    }

    fn get_url(&self, mt5_http_url: Mt5HttpUrl) -> Result<String, Mt5Error> {
        if let Some(port) = self.port {
            Ok(format!("{}:{}{}", Mt5HttpUrl::BaseUrl, port, mt5_http_url))
        } else {
            Err(HttpClientPortNotSetSnafu {
                terminal_id: self.terminal_id,
            }
            .build())
        }
    }

    #[instrument(skip(self))]
    pub async fn ping(&self) -> Result<(), Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::Ping)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5)) // timeout 5 seconds
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // if the status code is 200, then ping success
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                if is_success {
                    tracing::debug!("ping MT5 server success");
                    Ok(())
                } else {
                    let error_message = response_data
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    tracing::error!(error = %error_message, "failed to ping MT5 server");
                    return PingSnafu {
                        message: error_message,
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                return NoSuccessFieldInResponseSnafu {
                    terminal_id: self.terminal_id,
                    url: url.clone(),
                }
                .fail()?;
            }
        } else {
            // http status code is not 200, then ping failed
            let status_code = response.status().as_u16();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail()?;
        }
    }

    // 初始化MT5客户端
    #[instrument(skip(self, password, terminal_path), fields(login = %login, server = %server))]
    pub async fn initialize_terminal(&self, login: i64, password: &str, server: &str, terminal_path: &str) -> Result<(), Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::InitializeTerminal)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
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
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则初始化成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                if is_success {
                    tracing::info!(terminal_id = %login, "MT5 terminal initialized successfully");
                    return Ok(());
                } else {
                    // get error message
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    tracing::error!(error = %error_message, "failed to initialize MT5 terminal");
                    return InitializeTerminalFailedSnafu {
                        message: error_message,
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                return NoSuccessFieldInResponseSnafu {
                    terminal_id: self.terminal_id,
                    url: url.clone(),
                }
                .fail()?;
            }
        } else {
            let status_code = response.status().as_u16();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_terminal_info(&self) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetTerminalInfo)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        tracing::debug!(url = %url, "Getting terminal info");
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取终端信息
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
            // 判断是否有code
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                // 如果code为0，则返回data，否则返回错误
                if is_success {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!("Successfully got terminal info");
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get terminal info response success is {}", is_success))
                        .to_string();
                    tracing::error!(is_success = %is_success, error = %error_message, "Failed to get terminal info");
                    return GetTerminalInfoSnafu {
                        message: error_message,
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to get terminal info");
                return GetTerminalInfoSnafu {
                    message: error_message,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_symbol_list(&self) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetSymbolList)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        tracing::debug!(url = %url, "Getting symbol list");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                if is_success {
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    Ok(data.clone())
                } else {
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    return GetSymbolListSnafu {
                        message: error_message,
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                let error_message = "No success field in the response".to_string();
                return GetSymbolListSnafu {
                    message: error_message,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        } else {
            let status_code = response.status().as_u16();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_symbol_info(&self, symbol: &str) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetSymbolInfo)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        tracing::debug!(url = %url, symbol = %symbol, "Getting symbol info");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                if is_success {
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    Ok(data.clone())
                } else {
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    return GetSymbolInfoSnafu {
                        message: error_message,
                        symbol: symbol.to_string(),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                let error_message = "No success field in the response".to_string();
                return GetSymbolInfoSnafu {
                    message: error_message,
                    symbol: symbol.to_string(),
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        } else {
            let status_code = response.status().as_u16();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail();
        }
    }

    // 获取K线系列
    #[instrument(skip(self))]
    pub async fn get_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, limit: u32) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetKlineSeries)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?symbol={}&interval={}&limit={}", url, symbol, interval, limit);
        tracing::debug!(url = %url, symbol = %symbol, interval = %interval, limit = %limit, "Getting kline series");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;
        tracing::debug!("response: {:?}", response.status());

        // 如果为200，则获取K线数据成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            // 判断是否有code
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                // 如果code为0，则返回data，否则返回错误
                if is_success {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(symbol = %symbol, "Successfully got kline series");
                    Ok(data.clone())
                } else {
                    // if is_success is false, then return error
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    let error_code = response_data.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                    tracing::error!(code = %error_code, error = %error_message, symbol = %symbol, "Failed to get kline series");
                    return GetKlineDataSnafu {
                        symbol: symbol.to_string(),
                        message: error_message,
                        code: Some(error_code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无success错误"
                let error_message = "No success field in the response".to_string();
                tracing::error!(error = %error_message, symbol = %symbol, "The response has no success field");
                return GetKlineDataSnafu {
                    symbol: symbol.to_string(),
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, symbol = %symbol, "Failed to get kline series - HTTP error");
            return GetKlineDataSnafu {
                symbol: symbol.to_string(),
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    // 获取历史
    #[instrument(skip(self, time_range), fields(
        start_time = %time_range.start_date.to_utc().format("%Y-%m-%d %H:%M:%S").to_string(), 
        end_time = %time_range.end_date.to_utc().format("%Y-%m-%d %H:%M:%S").to_string()))
        ]
    pub async fn get_kline_history(
        &self,
        symbol: &str,
        interval: Mt5KlineInterval,
        time_range: TimeRange,
    ) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetKlineHistory)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let start_time = time_range.start_date.to_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let end_time = time_range.end_date.to_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        let url = format!(
            "{}?symbol={}&interval={}&start_time={}&end_time={}",
            url, symbol, interval, start_time, end_time
        );
        tracing::debug!("get kline history. url: {}", url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取K线数据成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            // if success is true, then return data, otherwise return error
            if let Some(is_success) = response_data.get("success").and_then(|v| v.as_bool()) {
                // 如果code为0，则返回data，否则返回错误
                if is_success {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    tracing::debug!(symbol = %symbol, "Successfully got kline history");
                    Ok(data.clone())
                } else {
                    // if success is false, then return error
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    let error_code = response_data.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                    tracing::error!(code = %error_code, error = %error_message, symbol = %symbol, "Failed to get kline history");
                    return GetKlineDataSnafu {
                        symbol: symbol.to_string(),
                        message: error_message,
                        code: Some(error_code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // if there is no success field, then return error
                let error_message = "No success field in the response".to_string();
                tracing::error!(error = %error_message, symbol = %symbol, "The response has no success field");
                return GetKlineDataSnafu {
                    symbol: symbol.to_string(),
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // if the status code is not 200, then return error
        else {
            let status_code = response.status();
            tracing::error!(status = %status_code, symbol = %symbol, "Failed to get kline history - HTTP error");
            // let error = response.error_for_status().unwrap_err();
            return ServerSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
                status_code: status_code,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn create_order(&self, params: Mt5CreateOrderParams) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::CreateOrder)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let symbol = &params.symbol;
        tracing::debug!(url = %url, params = ?params, "Creating order");

        let response = self
            .client
            .post(&url)
            .json(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则创建订单成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the create order response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to create order");
                    return CreateOrderSnafu {
                        symbol: symbol.to_string(),
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to create order");
                return CreateOrderSnafu {
                    symbol: symbol.to_string(),
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, "Failed to create order - HTTP error");
            return CreateOrderSnafu {
                symbol: symbol.to_string(),
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    // 获取订单
    #[instrument(skip(self))]
    pub async fn get_order(&self, order_id: &i64) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetOrder)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?order_id={}", url, order_id);
        tracing::debug!(url = %url, order_id = %order_id, "Getting order");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取订单成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get order response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, order_id = %order_id, "Failed to get order");
                    return GetOrderSnafu {
                        order_id: *order_id,
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, order_id = %order_id, "Failed to get order");
                return GetOrderSnafu {
                    order_id: *order_id,
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, order_id = %order_id, "Failed to get order - HTTP error");
            return GetOrderSnafu {
                order_id: *order_id,
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_position(&self, position_id: &i64) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetPosition)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?position_id={}", url, position_id);
        tracing::debug!(url = %url, position_id = %position_id, "Getting position");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取仓位成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get position response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, position_id = %position_id, "Failed to get position");
                    return GetPositionSnafu {
                        position_id: *position_id,
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, position_id = %position_id, "Failed to get position");
                return GetPositionSnafu {
                    position_id: *position_id,
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, position_id = %position_id, "Failed to get position - HTTP error");
            return GetPositionSnafu {
                position_id: *position_id,
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    // 获取成交明细
    #[instrument(skip(self))]
    pub async fn get_deal_by_position_id(&self, position_id: &i64) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetDeal)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?position_id={}", url, position_id);
        tracing::debug!(url = %url, position_id = %position_id, "Getting deal by position id");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get deal response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, position_id = %position_id, "Failed to get deal by position id");
                    return GetDealByPositionIdSnafu {
                        position_id: *position_id,
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, position_id = %position_id, "Failed to get deal by position id");
                return GetDealByPositionIdSnafu {
                    position_id: *position_id,
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, position_id = %position_id, "Failed to get deal by position id - HTTP error");
            return GetDealByPositionIdSnafu {
                position_id: *position_id,
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_deal_by_deal_id(&self, deal_id: &i64) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetDeal)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?deal_id={}", url, deal_id);
        tracing::debug!(url = %url, deal_id = %deal_id, "Getting deal by deal id");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get deal response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, deal_id = %deal_id, "Failed to get deal by deal id");
                    return GetDealByDealIdSnafu {
                        deal_id: *deal_id,
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, deal_id = %deal_id, "Failed to get deal by deal id");
                return GetDealByDealIdSnafu {
                    deal_id: *deal_id,
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, deal_id = %deal_id, "Failed to get deal by deal id - HTTP error");
            return GetDealByDealIdSnafu {
                deal_id: *deal_id,
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_deals_by_order_id(&self, order_id: &i64) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetDeal)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!("{}?order_id={}", url, order_id);
        tracing::debug!(url = %url, order_id = %order_id, "Getting deals by order id");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取成交明细成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get deals response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, order_id = %order_id, "Failed to get deals by order id");
                    return GetDealByOrderIdSnafu {
                        order_id: *order_id,
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, order_id = %order_id, "Failed to get deals by order id");
                return GetDealByOrderIdSnafu {
                    order_id: *order_id,
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, order_id = %order_id, "Failed to get deals by order id - HTTP error");
            return GetDealByOrderIdSnafu {
                order_id: *order_id,
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_position_number(&self, position_number_request: Mt5GetPositionNumberParams) -> Result<serde_json::Value, Mt5Error> {
        let symbol = &position_number_request.symbol;
        let position_side = position_number_request.position_side;

        let url = self.get_url(Mt5HttpUrl::GetPositionNumber)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;
        let url = format!(
            "{}?symbol={}{}",
            url,
            symbol,
            position_side
                .clone()
                .map_or(String::new(), |side| format!("&position_side={}", side))
        );

        tracing::debug!(url = %url, symbol = %symbol, position_side = ?position_side, "Getting position number");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取仓位数量成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

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
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get position number response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, symbol = %symbol, position_side = ?position_side, "Failed to get position number");
                    return GetPositionNumberSnafu {
                        symbol: symbol.to_string(),
                        message: error_message,
                        code: Some(code),
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, symbol = %symbol, position_side = ?position_side, "Failed to get position number");
                return GetPositionNumberSnafu {
                    symbol: symbol.to_string(),
                    message: error_message,
                    code: None,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, symbol = %symbol, position_side = ?position_side, "Failed to get position number - HTTP error");
            return GetPositionNumberSnafu {
                symbol: symbol.to_string(),
                message: format!("status code: {}, error text: {}", status_code, error_text),
                code: None,
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }

    #[instrument(skip(self))]
    pub async fn get_account_info(&self) -> Result<serde_json::Value, Mt5Error> {
        let url = self.get_url(Mt5HttpUrl::GetAccountInfo)?;
        let port = self.port.context(HttpClientPortNotSetSnafu {
            terminal_id: self.terminal_id,
        })?;

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context(NetworkSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

        // 如果为200，则获取账户信息成功
        if response.status().is_success() {
            let response_data = response.json::<serde_json::Value>().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            // 判断是否有code
            if let Some(code) = response_data.get("code").and_then(|v| v.as_i64()) {
                // 如果code为0，则返回data，否则返回错误
                if code == 0 {
                    // 如果有data，则返回data，否则返回null
                    let data = response_data.get("data").unwrap_or(&serde_json::Value::Null);
                    Ok(data.clone())
                } else {
                    // code不为0，则返回message
                    let error_message = response_data
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&format!("unknown error, the get account info response code is {}", code))
                        .to_string();
                    tracing::error!(code = %code, error = %error_message, "Failed to get account info");
                    return GetAccountInfoSnafu {
                        message: error_message,
                        terminal_id: self.terminal_id,
                        port,
                    }
                    .fail()?;
                }
            } else {
                // 没有code，则返回"无code错误"
                let error_message = "No code field in the response".to_string();
                tracing::error!(error = %error_message, "Failed to get account info");
                return GetAccountInfoSnafu {
                    message: error_message,
                    terminal_id: self.terminal_id,
                    port,
                }
                .fail()?;
            }
        }
        // 如果为其他状态码，则返回错误
        else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.context(ResponseSnafu {
                terminal_id: self.terminal_id,
                url: url.clone(),
            })?;

            tracing::error!(status = %status_code, error = %error_text, "Failed to get account info - HTTP error");
            return GetAccountInfoSnafu {
                message: format!("status code: {}, error text: {}", status_code, error_text),
                terminal_id: self.terminal_id,
                port,
            }
            .fail()?;
        }
    }
}
