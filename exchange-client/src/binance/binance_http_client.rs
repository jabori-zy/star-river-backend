#![allow(dead_code, unused_imports)]
use reqwest::{self, Response};
use serde_json::json;
use crate::utils::get_utc8_timestamp;
use crate::binance::url::BinanceHttpUrl;


use crate::utils::deserialize_string_to_f64;
// 导入lib.rs中的Kline和TickerPrice
use crate::binance::{BinanceKlineInterval, BinanceKline, BinanceTickerPrice};
use types::market::{Kline, TickerPrice};

#[derive(Clone)]

pub struct BinanceHttpClient {
    client: reqwest::Client,
    is_connected: bool,
    
}



impl BinanceHttpClient {
    pub fn new() -> Self {

        Self {
            client: reqwest::Client::new(),
            is_connected: false,

        }
    }

    pub async fn ping(&mut self) -> Result<(), String> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::Ping);

        let response = self.client.get(&url).send().await.expect("ping失败");
        let body = response.text().await.expect("ping失败");
        // 如果body为空，则认为连接成功
        if body == "{}" {
            self.is_connected = true;
        }
        Ok(())

    }





    pub async fn get_server_time(&self) -> Result<i64, String> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ServerTime);
        let response = self.client.get(&url).send().await.expect("获取服务器时间失败");
        let body = response.text().await.expect("获取服务器时间失败");
        

        // 解析JSON字符串
        let result: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("解析服务器时间失败: {}", e))?;
        
        // 提取时间戳
        result.get("serverTime")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| "无法获取服务器时间戳".to_string())
    }


    pub async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String> {
        // 构建url
        let url = format!("{}{}?symbol={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::PriceTicker, symbol);



        // 获取ticker price
        let mut tick_price = self.client
            .get(&url)
            .header("X-MBX-TIME-UNIT", "MILLISECOND")
            .send()
            .await.expect("获取ticker price失败")
            .json::<serde_json::Value>()
            .await.expect("解析ticker price失败");
        // 设置时间戳
        tick_price["timestamp"] = serde_json::Value::Number(get_utc8_timestamp().into());

        Ok(tick_price)
    }


    pub async fn get_kline(&self, 
        symbol: &str, 
        interval: BinanceKlineInterval, 
        limit: Option<u32>, 
        start_time: Option<u64>, 
        end_time: Option<u64>
    ) -> Result<Vec<serde_json::Value>, String> {
        // 如果limit为空，则设置为1000
        let limit = limit.unwrap_or(1000);
        // 如果start_time或end_time为空，则不传时间参数
        
        let url = if start_time.is_none() || end_time.is_none() {   
            format!("{}{}?symbol={}&interval={}&limit={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::Kline, symbol, interval, limit)
        } else {
            format!("{}{}?symbol={}&interval={}&limit={}&startTime={}&endTime={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::Kline, symbol, interval, limit,start_time.unwrap(), end_time.unwrap())
        };

        let raw_kline = self.client
            .get(&url)
            .send()
            .await
            .expect("获取k线数据失败")
            .json::<Vec<serde_json::Value>>()
            .await
            .expect("解析k线数据失败: {}");


        // log::debug!("kline: {:?}", raw_kline);

        Ok(raw_kline)
    }

    pub async fn get_exchange_info(&self) -> Result<String, String> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ExchangeInfo);
        let response = self.client.get(&url).send().await.expect("获取交易所信息失败");
        let body = response.text().await.expect("获取交易所信息失败");
        

        // let result: serde_json::Value = serde_json::from_str(&body)
        //     .map_err(|e| format!("解析交易所信息失败: {}", e))?;


        Ok(body)

    }
}









