use crate::metatrader5::url::Mt5HttpUrl;
use serde::Serialize;


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

    pub async fn get_latest_kline(&mut self, symbol: &str, time_frame: &str) -> Result<(), String> {
        let url = format!("{}{}?symbol={}&time_frame={}", Mt5HttpUrl::BaseUrl, Mt5HttpUrl::GetLatestKline, symbol, time_frame);
        let response = self.client.get(&url).send().await.expect("获取最新K线失败");
        let body = response.text().await.expect("获取最新K线失败");
        tracing::info!("最新K线: {}", body);
        Ok(())
    }
}




