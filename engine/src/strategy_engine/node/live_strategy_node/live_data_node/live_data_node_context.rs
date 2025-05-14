use event_center::command_event::CommandEvent;
use types::market::KlineInterval;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_message::{KlineSeriesMessage, NodeMessage};
use uuid::Uuid;
use event_center::response_event::ResponseEvent;
use crate::strategy_engine::node::node_context::{NodeContext,BaseNodeContext};
use event_center::command_event::market_engine_command::{MarketEngineCommand, SubscribeKlineStreamParams, UnsubscribeKlineStreamParams};
use event_center::command_event::exchange_engine_command::RegisterExchangeParams;
use event_center::response_event::market_engine_response::MarketEngineResponse;
use event_center::response_event::exchange_engine_response::ExchangeEngineResponse;
use event_center::command_event::exchange_engine_command::ExchangeEngineCommand;
use types::strategy::SelectedAccount;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use heartbeat::Heartbeat;
use event_center::command_event::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::cache::{CacheKey, cache_key::KlineCacheKey};
use event_center::EventPublisher;
use event_center::response_event::cache_engine_response::CacheEngineResponse;
use crate::strategy_engine::node::node_types::NodeOutputHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_kline_interval")]
    pub interval: KlineInterval,
    // pub frequency: u32,
}

fn deserialize_kline_interval<'de, D>(deserializer: D) -> Result<KlineInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;
    
    // 使用as_str()方法获取&str，然后传递给from_str
    match KlineInterval::from_str(s.as_str()) {
        Ok(interval) => Ok(interval),
        Err(e) => Err(serde::de::Error::custom(format!("无法解析KlineInterval: {}", e)))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeSimulateConfig {
    pub selected_simulate_accounts: SelectedAccount,
    pub symbol: String,
    pub interval: KlineInterval,
    // pub frequency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeBacktestConfig {
    pub start_date: String,
    pub end_date: String,
    pub accounts: Vec<i32>,
}



#[derive(Debug, Clone)]
pub struct LiveDataNodeContext {
    pub base_context: BaseNodeContext,
    pub stream_is_subscribed: Arc<RwLock<bool>>,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub request_ids: Arc<Mutex<Vec<Uuid>>>,
    pub live_config: LiveDataNodeLiveConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}

#[async_trait]
impl NodeContext for LiveDataNodeContext {

    fn clone_box(&self) -> Box<dyn NodeContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("live_data_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        match event {
            // Event::Market(market_event) => {
            //     self.handle_market_event(market_event).await;
            // }
            Event::Response(response_event) => {
                self.handle_response_event(response_event).await;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
    
}


impl LiveDataNodeContext {

    async fn remove_request_id(&mut self, request_id: Uuid) {
        let mut request_id_guard = self.request_ids.lock().await;
        let index = request_id_guard.iter().position(|id| *id == request_id).unwrap();
        request_id_guard.remove(index);
    }

    async fn handle_response_event(&mut self, response_event: ResponseEvent) {
        // tracing::info!("{}: 收到响应事件: {:?}", self.base_context.node_id, response_event);
        // 注册交易所的响应
        match response_event {
            ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeResponse(register_exchange_success_response)) => {
                let contains = {
                    let request_id_guard = self.request_ids.lock().await;
                    request_id_guard.contains(&register_exchange_success_response.response_id)
                };

                if contains {
                    self.remove_request_id(register_exchange_success_response.response_id).await;
                    // 接收到交易所注册成功的消息，修改订阅状态为true
                    *self.exchange_is_registered.write().await = true;
                    tracing::info!("exchange_is_registered: {:?}", self.exchange_is_registered.read().await);
                    tracing::info!("{}: 交易所注册成功: {:?}", self.base_context.node_id, register_exchange_success_response);
                }
            }
            // 订阅k线流的响应
            ResponseEvent::MarketEngine(MarketEngineResponse::SubscribeKlineStreamSuccess(subscribe_kline_stream_success_response)) => {
                let contains = {
                    let request_id_guard = self.request_ids.lock().await;
                    request_id_guard.contains(&subscribe_kline_stream_success_response.response_id)
                };

                if contains {
                    self.remove_request_id(subscribe_kline_stream_success_response.response_id).await;
                    tracing::info!("{}: K线流订阅成功: {:?}, 开始推送数据", self.base_context.node_id, subscribe_kline_stream_success_response);
                    // 接收到stream订阅成功的消息，修改订阅状态为true
                    *self.stream_is_subscribed.write().await = true;
                    
                    tracing::warn!("{}: 订阅状态修改为true", self.base_context.node_id);
                }
            }
            // 取消订阅k线流的响应
            ResponseEvent::MarketEngine(MarketEngineResponse::UnsubscribeKlineStreamSuccess(unsubscribe_kline_stream_success_response)) => {
                let contains = {
                    let request_id_guard = self.request_ids.lock().await;
                    request_id_guard.contains(&unsubscribe_kline_stream_success_response.response_id)
                };

                if contains {
                    self.remove_request_id(unsubscribe_kline_stream_success_response.response_id).await;
                    tracing::info!("{}: K线流取消订阅成功: {:?}, 停止推送数据", self.base_context.node_id, unsubscribe_kline_stream_success_response);
                    // 修改订阅状态为false
                    *self.stream_is_subscribed.write().await = false;
                }
            }
            // 获取k线数据的响应
            ResponseEvent::CacheEngine(CacheEngineResponse::GetCacheData(get_cache_data_response)) => {
                let contains = {
                    let request_id_guard = self.request_ids.lock().await;
                    request_id_guard.contains(&get_cache_data_response.response_id)
                };

                if contains {
                    self.remove_request_id(get_cache_data_response.response_id).await;
                    let kline_series_message = KlineSeriesMessage {
                        from_node_id: self.base_context.node_id.clone(),
                        from_node_name: self.base_context.node_name.clone(),
                        exchange: get_cache_data_response.cache_key.get_exchange(),
                        symbol: get_cache_data_response.cache_key.get_symbol(),
                        interval: get_cache_data_response.cache_key.get_interval(),
                        kline_series: get_cache_data_response.cache_data,
                        message_timestamp: get_utc8_timestamp_millis(),
                    };

                    let message = NodeMessage::KlineSeries(kline_series_message);
                    // tracing::debug!("{}: 发送数据: {:?}", self.base_context.node_id, message);
                    let output_handle = self.get_default_output_handle();
                    output_handle.send(message).unwrap();
                }
            }
            _ => {}
        }
    }

    pub async fn register_exchange(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始注册交易所", self.base_context.node_name);
        let request_id = Uuid::new_v4();
        let register_param = RegisterExchangeParams {
            account_id: self.live_config.selected_live_account.account_id.clone(),
            exchange: self.live_config.selected_live_account.exchange.clone(),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };


        let mut request_id_guard = self.request_ids.lock().await;
        request_id_guard.push(request_id);
        drop(request_id_guard);
        tracing::warn!("{}: 注册交易所的请求id: {:?}", self.base_context.node_name, self.request_ids);

        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterExchange(register_param));
        tracing::info!("{}注册交易所: {:?}", self.base_context.node_id, command_event);
        if let Err(e) = self.base_context.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_context.node_id,
                error = ?e,
                "数据源节点发送注册交易所失败"
            );
        }
        Ok(())
        
        
    }

    pub async fn subscribe_kline_stream(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();

        let params = SubscribeKlineStreamParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: self.live_config.selected_live_account.account_id.clone(),
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            frequency: 1000,
            cache_size: 500,
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };


        let mut request_id_guard = self.request_ids.lock().await;
        request_id_guard.push(request_id);
        drop(request_id_guard);

        let command_event = CommandEvent::MarketEngine(MarketEngineCommand::SubscribeKlineStream(params));
        tracing::info!("{}订阅k线流: {:?}", self.base_context.node_name, command_event);
        if let Err(e) = self.get_event_publisher().publish(command_event.into()) {
            tracing::error!(
                node_name = %self.base_context.node_name,
                error = ?e,
                "数据源节点订阅k线流失败"
            );
        }
        Ok(())
    }

    pub async fn unsubscribe_kline_stream(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();
        let (account_id, exchange, symbol, interval) = (
            self.live_config.selected_live_account.account_id.clone(), 
            self.live_config.selected_live_account.exchange.clone(), 
            self.live_config.symbol.clone(), 
            self.live_config.interval.clone());

        let params = UnsubscribeKlineStreamParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: account_id,
            exchange: exchange,
            symbol: symbol,
            interval: interval,
            frequency: 1000,
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        // 设置请求id
        let mut request_id_guard = self.request_ids.lock().await;
        request_id_guard.push(request_id);
        drop(request_id_guard);

        let command_event = CommandEvent::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(params));
        tracing::debug!("{}取消订阅k线流: {:?}", self.base_context.node_name, command_event);
        if let Err(_) = self.base_context.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_context.node_id,
                "数据源节点发送数据失败"
            );
        }   
        Ok(())
    }

    pub async fn register_task(&mut self) {
        let node_name = self.base_context.node_name.clone();
        let kline_cache_key = CacheKey::Kline(KlineCacheKey {
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
        });
        let event_publisher = self.get_event_publisher().clone();
        let strategy_id = self.base_context.strategy_id.clone();
        let node_id = self.base_context.node_id.clone();
        let request_ids = self.request_ids.clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            format!("{}获取k线数据", node_name),
            move || {
                let kline_cache_key = kline_cache_key.clone();
                let event_publisher = event_publisher.clone();
                let strategy_id = strategy_id.clone();
                let node_id = node_id.clone();
                let request_ids = request_ids.clone();
                async move {
                    Self::get_kline_series_cache(
                        strategy_id,
                        node_id,
                        kline_cache_key,
                        20,
                        event_publisher,
                        request_ids,
                    ).await;
                }
            },
            10
        ).await;
    }

    // 从缓存引擎获取k线数据
    pub async fn get_kline_series_cache(
        strategy_id: i32, 
        node_id: String, 
        kline_cache_key: CacheKey, 
        limit: u32,
        event_publisher: EventPublisher,
        request_ids: Arc<Mutex<Vec<Uuid>>>,
    ){
        let request_id = Uuid::new_v4();
        let params = GetCacheParams {
            strategy_id: strategy_id,
            node_id: node_id.clone(),
            cache_key: kline_cache_key,
            limit: Some(limit),
            sender: node_id,
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        let mut request_id_guard = request_ids.lock().await;
        request_id_guard.push(request_id);
        drop(request_id_guard);
        let command_event = CommandEvent::CacheEngine(CacheEngineCommand::GetCache(params));
        let _ = event_publisher.publish(command_event.into());
    }

}