use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use async_trait::async_trait;
use event_center::Event;
use event_center::command::Command;
use event_center::response::Response;
use event_center::response::indicator_engine_response::IndicatorEngineResponse;
use event_center::command::indicator_engine_command::{IndicatorEngineCommand, RegisterIndicatorParams};
use utils::get_utc8_timestamp_millis;
use types::strategy::node_message::{IndicatorMessage, NodeMessage};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use event_center::command::cache_engine_command::{AddCacheKeyParams, CacheEngineCommand, GetCacheParams};
use event_center::command::indicator_engine_command::CalculateBacktestIndicatorParams;
use types::cache::cache_key::IndicatorCacheKey;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::cache::CacheValue;
use tokio::sync::oneshot;
use event_center::response::ResponseTrait;
use super::indicator_node_type::IndicatorNodeBacktestConfig;
use types::cache::cache_key::{BacktestIndicatorCacheKey, BacktestKlineCacheKey};
use tokio::time::Duration;
use types::indicator::IndicatorConfig;
use types::indicator::Indicator;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    pub is_registered: Arc<RwLock<bool>>, // 是否已经注册指标
}




#[async_trait]
impl BacktestNodeContextTrait for IndicatorNodeContext {
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("indicator_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {



        Ok(())
    }

    
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::BacktestKlineUpdate(backtest_kline_message) => {
                // tracing::info!("节点{}收到回测K线更新: {:?}", self.base_context.node_id, backtest_kline_message);
                let indicator_cache_data = self.get_backtest_indicator_cache(backtest_kline_message.kline_cache_index).await.unwrap();
                // 将K线数据转换为IndicatorValue
                let indicator_value = indicator_cache_data.into_iter().map(|cache_value| cache_value.as_indicator().unwrap()).collect::<Vec<Indicator>>();
                tracing::info!("节点{}收到回测指标数据: {:?}, index: {}", self.base_context.node_id, indicator_value, backtest_kline_message.kline_cache_index);


            }
            _ => {}

        }
        Ok(())
    }

}

impl IndicatorNodeContext {


    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator_cache_key(&self) -> Result<Response, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        // 创建
        let cache_key = BacktestIndicatorCacheKey {
            exchange: self.backtest_config.exchange_config.clone().unwrap().exchange, 
            symbol: self.backtest_config.exchange_config.clone().unwrap().symbol, 
            interval: self.backtest_config.exchange_config.clone().unwrap().interval, 
            indicator_config: self.backtest_config.indicator_config.clone(),
            start_time: self.backtest_config.exchange_config.clone().unwrap().time_range.start_date.to_string(),
            end_time: self.backtest_config.exchange_config.clone().unwrap().time_range.end_date.to_string(),
        };
        let register_indicator_params = AddCacheKeyParams {
            strategy_id: self.base_context.strategy_id.clone(),
            cache_key: cache_key.into(),
            max_size: None,
            duration: Duration::from_secs(30),
            sender: self.base_context.node_id.to_string(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };
        let register_indicator_command = Command::CacheEngine(CacheEngineCommand::AddCacheKey(register_indicator_params));
        self.get_command_publisher().send(register_indicator_command).await.unwrap();

        // 等待响应
        let register_indicator_response = resp_rx.await.unwrap();
        Ok(register_indicator_response)
    }


    // 获取已经计算好的回测指标数据
    async fn get_backtest_indicator_cache(&self, index: u32) -> Result<Vec<Arc<CacheValue>>, String> {
        let indicator_cache_key = BacktestIndicatorCacheKey {
            exchange: self.backtest_config.exchange_config.clone().unwrap().exchange,
            symbol: self.backtest_config.exchange_config.clone().unwrap().symbol,
            interval: self.backtest_config.exchange_config.clone().unwrap().interval,
            indicator_config: self.backtest_config.indicator_config.clone(),
            start_time: self.backtest_config.exchange_config.clone().unwrap().time_range.start_date.to_string(),
            end_time: self.backtest_config.exchange_config.clone().unwrap().time_range.end_date.to_string(),
        };
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            cache_key: indicator_cache_key.into(),
            index: Some(index),
            limit: Some(1),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };
    
        let get_cache_command = CacheEngineCommand::GetCache(params);
        self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        tracing::info!("节点{}收到回测K线缓存数据: {:?}", self.base_context.node_id, get_cache_data_response.cache_data.len());
                        return Ok(get_cache_data_response.cache_data)
                    }
                    _ => {
                        return Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id))
                    }
                }
            }
        }
        Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id))
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<Response, String> {

        let kline_cache_key = BacktestKlineCacheKey {
                exchange: self.backtest_config.exchange_config.clone().unwrap().exchange,
                symbol: self.backtest_config.exchange_config.clone().unwrap().symbol,
                interval: self.backtest_config.exchange_config.clone().unwrap().interval,
                start_time: self.backtest_config.exchange_config.clone().unwrap().time_range.start_date.to_string(),
                end_time: self.backtest_config.exchange_config.clone().unwrap().time_range.end_date.to_string(),
            };
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = CalculateBacktestIndicatorParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            kline_cache_key: kline_cache_key.into(),
            indicator_config: self.backtest_config.indicator_config.clone(),
            sender: self.base_context.node_id.clone(),
            command_timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };
        let calculate_indicator_command = Command::IndicatorEngine(IndicatorEngineCommand::CalculateBacktestIndicator(params));
        self.get_command_publisher().send(calculate_indicator_command).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }
}

