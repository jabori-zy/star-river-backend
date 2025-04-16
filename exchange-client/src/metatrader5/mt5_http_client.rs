use crate::metatrader5::url::Mt5HttpUrl;
use crate::metatrader5::mt5_types::Mt5KlineInterval;
use serde::Serialize;

use super::mt5_types::Mt5PositionNumberRequest;
use super::mt5_types::Mt5CreateOrderParams;


#[derive(Debug)]
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

    pub async fn create_order(&self, params: Mt5CreateOrderParams) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::CreateOrder);
        tracing::debug!("metatrader5 创建订单请求: {:?}", params);

        let response = self.client.post(&url)
        .json(&params)
        .send()
        .await.expect("创建订单失败");

        let response_text = response.text().await.expect("创建订单失败");
        tracing::debug!("metatrader5 创建订单响应: {}", response_text);
        let response_json = serde_json::from_str::<serde_json::Value>(&response_text).expect("创建订单失败");
        Ok(response_json)
    }

    // 获取订单
    pub async fn get_order(&self, order_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}{}?order_id={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetOrder, order_id);
        let response = self.client.get(&url).send().await.expect("获取订单失败").json::<serde_json::Value>().await.expect("获取订单失败");
        Ok(response)
    }

    pub async fn get_position(&self, position_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}{}?position_id={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetPosition, position_id);
        let response = self.client.get(&url).send().await.expect("获取仓位失败").json::<serde_json::Value>().await.expect("获取仓位失败");
        Ok(response)
    }

    // 获取成交明细
    pub async fn get_deal_by_position_id(&self, position_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}{}?position_id={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetDeal, position_id);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }

    pub async fn get_deal_by_deal_id(&self, deal_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}{}?deal_id={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetDeal, deal_id);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }

    pub async fn get_deals_by_order_id(&self, order_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}{}?order_id={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetDeal, order_id);
        tracing::debug!("metatrader5 获取成交明细请求: {}", url);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }



    pub async fn get_position_number(&self, position_number_request: Mt5PositionNumberRequest) -> Result<serde_json::Value, String> {
        
        tracing::debug!("metatrader5 获取仓位数量: {:?}", position_number_request);
        let url = format!("{}{}?symbol={}{}",
            Mt5HttpUrl::BaseUrl,
            Mt5HttpUrl::GetPositionNumber,
            position_number_request.symbol,
            position_number_request.position_side
                .map_or(String::new(), |side| format!("&position_side={}", side))
        );
        
        let response = self.client.get(url).send().await.unwrap().json::<serde_json::Value>().await.expect("获取仓位数量失败");
        Ok(response)

    }

    pub async fn get_account_info(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetAccountInfo);
        let response = self.client.get(url).send().await.unwrap().json::<serde_json::Value>().await.expect("获取账户信息失败");
        Ok(response)
    }

}




