use crate::metatrader5::url::Mt5HttpUrl;
use crate::metatrader5::mt5_types::Mt5KlineInterval;
use serde::Serialize;

use super::mt5_types::Mt5PositionNumberRequest;
use super::mt5_types::Mt5CreateOrderParams;

#[derive(Debug)]
pub struct Mt5HttpClient {
    port: u16,
    client: reqwest::Client,
    is_connected: bool,
}


impl Mt5HttpClient {
    pub fn new(port: u16) -> Self {
        Self {
            port: port,
            client: reqwest::Client::new(),
            is_connected: false,
        }
    }

    fn get_url(&self, mt5_http_url: Mt5HttpUrl) -> String {
        format!("{}:{}{}", Mt5HttpUrl::BaseUrl, self.port, mt5_http_url)
    }

    pub async fn ping(&self) -> Result<serde_json::Value, String> {
        let url = self.get_url(Mt5HttpUrl::Ping);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;

        let response_text = response.text().await.map_err(|e| e.to_string())?;
        tracing::debug!("metatrader5 ping响应: {}", response_text);
        let response_json = serde_json::from_str::<serde_json::Value>(&response_text).map_err(|e| e.to_string())?;
        Ok(response_json)
    }

    // 初始化MT5客户端
    pub async fn initialize_terminal(
        &self,
        login: i64,
        password: &str,
        server: &str,
        terminal_path: &str
    ) -> Result<(), String> {
        let url = self.get_url(Mt5HttpUrl::InitializeTerminal);
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
        .send().await.expect("初始化失败");

        let body = response.text().await.expect("初始化失败");
        tracing::debug!("metatrader5 初始化响应: {}", body);
        Ok(())
    }



    pub async fn login(&self, login: i64, password: &str, server: &str) -> Result<serde_json::Value, String> {
        #[derive(Debug, Serialize)]
        struct LoginRequest {
            login: i64,
            password: String,
            server: String,
        }
        let request = LoginRequest {
            login: login,
            password: password.to_string(),
            server: server.to_string(),
        };
        let url = self.get_url(Mt5HttpUrl::Login);

        let response = self.client.post(&url)
        .json(&request)
        .send()
        .await.expect("登录失败");

        let response_text = response.text().await.expect("登录失败");
        tracing::debug!("metatrader5 登录响应: {}", response_text);
        let response_json = serde_json::from_str::<serde_json::Value>(&response_text).expect("登录失败");
        Ok(response_json)
    }


    pub async fn get_terminal_info(&self) -> Result<serde_json::Value, String> {
        let url = self.get_url(Mt5HttpUrl::GetTerminalInfo);
        let response = self.client.get(&url).send().await.expect("获取终端信息失败").json::<serde_json::Value>().await.expect("获取终端信息失败");
        Ok(response)
    }

    pub async fn get_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, limit: u32) -> Result<serde_json::Value, String> {
        let url = format!("{}?symbol={}&interval={}&limit={}", self.get_url(Mt5HttpUrl::GetKlineSeries), symbol, interval, limit);
        
        let response = self.client.get(&url)
        .send()
        .await.expect("获取K线系列失败")
        .json::<serde_json::Value>()
        .await.expect("获取K线系列失败");
        Ok(response)
    }

    pub async fn create_order(&self, params: Mt5CreateOrderParams) -> Result<serde_json::Value, String> {
        let url = self.get_url(Mt5HttpUrl::CreateOrder);
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
        let url = format!("{}?order_id={}", self.get_url(Mt5HttpUrl::GetOrder), order_id);
        let response = self.client.get(&url).send().await.expect("获取订单失败").json::<serde_json::Value>().await.expect("获取订单失败");
        Ok(response)
    }

    pub async fn get_position(&self, position_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}?position_id={}", self.get_url(Mt5HttpUrl::GetPosition), position_id);
        let response = self.client.get(&url).send().await.expect("获取仓位失败").json::<serde_json::Value>().await.expect("获取仓位失败");
        Ok(response)
    }

    // 获取成交明细
    pub async fn get_deal_by_position_id(&self, position_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}?position_id={}", self.get_url(Mt5HttpUrl::GetDeal), position_id);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }

    pub async fn get_deal_by_deal_id(&self, deal_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}?deal_id={}", self.get_url(Mt5HttpUrl::GetDeal), deal_id);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }

    pub async fn get_deals_by_order_id(&self, order_id: &i64) -> Result<serde_json::Value, String> {
        let url = format!("{}?order_id={}", self.get_url(Mt5HttpUrl::GetDeal), order_id);
        tracing::debug!("metatrader5 获取成交明细请求: {}", url);
        let response = self.client.get(&url).send().await.expect("获取成交明细失败").json::<serde_json::Value>().await.expect("获取成交明细失败");
        Ok(response)
    }



    pub async fn get_position_number(&self, position_number_request: Mt5PositionNumberRequest) -> Result<serde_json::Value, String> {
        
        tracing::debug!("metatrader5 获取仓位数量: {:?}", position_number_request);
        let url = format!("{}?symbol={}{}",
            self.get_url(Mt5HttpUrl::GetPositionNumber),
            position_number_request.symbol,
            position_number_request.position_side
                .map_or(String::new(), |side| format!("&position_side={}", side))
        );
        
        let response = self.client.get(url).send().await.unwrap().json::<serde_json::Value>().await.expect("获取仓位数量失败");
        Ok(response)

    }

    pub async fn get_account_info(&self) -> Result<serde_json::Value, String> {
        let url = self.get_url(Mt5HttpUrl::GetAccountInfo);
        let response = self.client.get(url).send().await.unwrap().json::<serde_json::Value>().await.expect("获取账户信息失败");
        Ok(response)
    }

}




