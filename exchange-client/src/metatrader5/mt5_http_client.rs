use crate::metatrader5::url::Mt5HttpUrl;
use crate::metatrader5::Mt5KlineInterval;
use serde::Serialize;
use types::order::{OrderType, OrderSide, OrderRequest};

pub struct Mt5HttpClient {
    client: reqwest::Client,
    is_connected: bool,
}


impl Mt5HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            is_connected: false,
        }
    }

    pub async fn ping(&mut self) -> Result<(), String> {
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::Ping);
        let response = self.client.get(&url).send().await.expect("ping失败");
        let body = response.text().await.expect("ping失败");
        Ok(())
    }

    // 初始化MT5客户端
    pub async fn initialize_client(&mut self) -> Result<(), String> {
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::InitializeClient);
        let response = self.client.get(&url).send().await.expect("初始化失败");
        let body = response.text().await.expect("初始化失败");
        tracing::info!("初始化响应: {}", body);
        Ok(())
    }

    pub async fn get_client_status(&mut self) -> Result<(), String> {
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::ClientStatus);
        let response = self.client.get(&url).send().await.expect("获取客户端状态失败");
        let body = response.text().await.expect("获取客户端状态失败");
        tracing::info!("客户端状态: {}", body);
        Ok(())
    }

    pub async fn login(&mut self, account_id: i32, password: &str, server: &str, terminal_path: &str) -> Result<(), String> {
        #[derive(Debug, Serialize)]
        struct LoginRequest {
            account_id: i32,
            password: String,
            server: String,
            terminal_path: String,
        }
        let request = LoginRequest {
            account_id: account_id,
            password: password.to_string(),
            server: server.to_string(),
            terminal_path: terminal_path.to_string(),
        };
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::Login);

        let response = self.client.post(&url)
        .json(&request)
        .send()
        .await.expect("登录失败")
        .json::<serde_json::Value>()
        .await.expect("登录失败");
        
        tracing::info!("Login response: {}", response);
        Ok(())
    }

    pub async fn get_kline_series(&mut self, symbol: &str, interval: Mt5KlineInterval, limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
        let limit = limit.unwrap_or(1000);
        let url = format!("{}{}?symbol={}&interval={}&limit={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetKlineSeries, symbol, interval, limit);
        
        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("获取K线系列请求失败: {}", e)),
        };

        if !response.status().is_success() {
            return Err(format!("获取K线系列失败, 状态码: {}", response.status()));
        }

        let response_json = match response.json::<serde_json::Value>().await {
            Ok(json) => json,
            Err(e) => return Err(format!("解析K线响应JSON失败: {}", e)),
        };

        // 检查状态码
        let status = response_json["status"].as_i64().unwrap_or(-1);
        if status != 0 {
            let message = response_json["message"].as_str().unwrap_or("未知错误");
            return Err(format!("获取K线失败: {}", message));
        }

        // 提取data字段
        match response_json["data"].as_array() {
            Some(data) => Ok(data.clone()),
            None => Err("K线数据格式错误".to_string()),
        }
    }

    pub async fn create_order(&mut self, order_request: OrderRequest) -> Result<serde_json::Value, String> {
        #[derive(Debug, Serialize)]
        struct CreateOrderRequest {
            order_type: String,
            order_side: String,
            symbol: String,
            volume: f64,
            price: f64,
            tp: Option<f64>,
            sl: Option<f64>,
        }
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::CreateOrder);
        let request = CreateOrderRequest {
            order_side: order_request.order_side.to_string(),
            order_type: order_request.order_type.to_string(),
            symbol: order_request.symbol.to_string(),
            volume: order_request.quantity,
            price: order_request.price,
            tp: order_request.tp,
            sl: order_request.sl,
        };

        let response = self.client.post(&url)
        .json(&request)
        .send()
        .await.expect("创建订单失败")
        .json::<serde_json::Value>()
        .await.expect("创建订单失败");
        Ok(response)
    }

}




