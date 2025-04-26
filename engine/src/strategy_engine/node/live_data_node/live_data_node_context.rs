use event_center::command_event::CommandEvent;
use types::market::{Exchange, KlineInterval};
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use event_center::market_event::MarketEvent;
use types::strategy::message::{KlineSeriesMessage, NodeMessage};
use uuid::Uuid;
use event_center::response_event::ResponseEvent;
use event_center::strategy_event::StrategyEvent;
use super::super::node_types::NodeRunState;
use super::super::node_context::{NodeContext,BaseNodeContext};
use event_center::command_event::market_engine_command::{MarketEngineCommand, SubscribeKlineStreamParams, UnsubscribeKlineStreamParams};
use event_center::command_event::exchange_engine_command::RegisterExchangeParams;
use event_center::response_event::market_engine_response::MarketEngineResponse;
use event_center::response_event::exchange_engine_response::ExchangeEngineResponse;
use event_center::command_event::exchange_engine_command::ExchangeEngineCommand;
use types::strategy::SelectedAccount;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use types::strategy::TradeMode;

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
    pub account_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub is_subscribed: bool,
    pub request_id: Option<Uuid>,
    pub live_config: Option<LiveDataNodeLiveConfig>,
    pub backtest_config: Option<LiveDataNodeBacktestConfig>,
    pub simulated_config: Option<LiveDataNodeSimulateConfig>,
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

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Market(market_event) => {
                self.handle_market_event(market_event).await;
            }
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

    async fn handle_market_event(&self, market_event: MarketEvent) {
        // 先获取读锁，检查状态
        // let state_guard = self.base_state.clone();

        if self.base_context.state_machine.current_state() != NodeRunState::Running {
            tracing::warn!("{}: 节点状态不是Running, 不处理行情数据", self.base_context.node_id);
            return;
        }

        // 处理市场事件
        match market_event {
            MarketEvent::KlineSeriesUpdate(kline_series_update) => {
                // 只获取当前节点支持的数据
                let exchange = self.exchange.clone();
                let symbol = self.symbol.clone();
                let interval = self.interval.clone();
                if exchange != kline_series_update.exchange || symbol != kline_series_update.symbol || interval != kline_series_update.interval {
                    return;
                }
                // 这里不需要再获取锁，因为我们只需要读取数据
                let kline_series_message = KlineSeriesMessage {
                    from_node_id: self.base_context.node_id.clone(),
                    from_node_name: self.base_context.node_name.clone(),
                    exchange: kline_series_update.exchange,
                    symbol: kline_series_update.symbol,
                    interval: kline_series_update.interval,
                    kline_series: kline_series_update.kline_series.clone(),
                    batch_id: kline_series_update.batch_id.clone(),
                    message_timestamp: get_utc8_timestamp_millis(),
                };
                
                let message = NodeMessage::KlineSeries(kline_series_message);
                // tracing::debug!("{}: 发送数据: {:?}", self.base_context.node_id, message);
                // 获取handle的连接数
                let default_handle_connect_count = self.base_context.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在").sender.receiver_count();
                // 如果连接数为0，则不发送数据
                if default_handle_connect_count > 0 {
                    let default_node_sender = self.base_context.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在");
                    // tracing::info!("{}: 发送数据: {:?}", state_guard.node_id, message);
                    match default_node_sender.sender.send(message.clone()) {
                        Ok(_) => (),
                        Err(e) => tracing::error!(
                            node_id = %self.base_context.node_id,
                            error = ?e,
                            receiver_count = default_node_sender.sender.receiver_count(),
                                "数据源节点发送数据失败"
                            ),
                        }
                    
                }

                // 发送事件
                if self.is_enable_event_publish().clone() {
                    let event = Event::Strategy(StrategyEvent::NodeMessage(message));
                    if let Err(_) = self.get_event_publisher().publish(event.into()) {
                        tracing::error!(
                            node_id = %self.base_context.node_id,
                            "数据源节点发送数据事件失败"
                        );
                    }
                }

            }
            _ => {}
        }
    }

    async fn handle_response_event(&mut self, response_event: ResponseEvent) {
        // tracing::info!("{}: 收到响应事件: {:?}", self.base_context.node_id, response_event);
        let request_id= {
            match self.request_id {
                Some(id) => {
                    id
                },
                None => {
                    return;
                }
            }
        };


        match response_event {
            ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(register_exchange_success_response)) => {
                if request_id == register_exchange_success_response.response_id {
                    tracing::info!("{}: 交易所注册成功: {:?}", self.base_context.node_id, register_exchange_success_response);
                    self.request_id = None;
                }
            }
            ResponseEvent::MarketEngine(MarketEngineResponse::SubscribeKlineStreamSuccess(subscribe_kline_stream_success_response)) => {
                
                if request_id == subscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流订阅成功: {:?}, 开始推送数据", self.base_context.node_id, subscribe_kline_stream_success_response);
                    self.request_id = None;
                    // 修改订阅状态为true
                    self.is_subscribed = true;
                    tracing::warn!("{}: 订阅状态修改为true", self.base_context.node_id);
                }
            }
            ResponseEvent::MarketEngine(MarketEngineResponse::UnsubscribeKlineStreamSuccess(unsubscribe_kline_stream_success_response)) => {
                if request_id == unsubscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流取消订阅成功: {:?}, 停止推送数据", self.base_context.node_id, unsubscribe_kline_stream_success_response);
                    self.request_id = None;
                    // 修改订阅状态为false
                    self.is_subscribed = false;
                }
            }   
            _ => {}
        }
    }

    pub async fn register_exchange(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();
        let register_param = match self.base_context.trade_mode {
            // 实盘模式
            TradeMode::Live => {
                tracing::info!("{}: 实盘模式", self.base_context.node_name);
                match self.live_config.clone() {
                    Some(config) => {
                        let register_param = RegisterExchangeParams {
                            account_id: config.selected_live_account.account_id.clone(),
                            exchange: config.selected_live_account.exchange.clone(),
                            sender: self.base_context.node_id.clone(),
                            timestamp: get_utc8_timestamp_millis(),
                            request_id: request_id,
                        };
                        register_param
                    },
                    None => {
                        return Err("实盘模式未配置".to_string());
                    }
                }
            },
            // 回测模式
            TradeMode::Backtest => {
                match self.backtest_config.clone() {
                    Some(backtest_config) => {  
                        let register_param = RegisterExchangeParams {
                            account_id: 1,
                            exchange: self.exchange.clone(),
                            sender: self.base_context.node_id.clone(),
                            timestamp: get_utc8_timestamp_millis(),
                            request_id: request_id,
                        };
                        register_param
                    },
                    None => {
                        return Err("回测模式未配置".to_string());
                    }
                }
            },
            // 模拟模式
            TradeMode::Simulated => {
                match self.simulated_config.clone() {
                    Some(simulated_config) => {
                        let register_param = RegisterExchangeParams {
                            account_id: simulated_config.selected_simulate_accounts.account_id.clone(),
                            exchange: self.exchange.clone(),
                            sender: self.base_context.node_id.clone(),
                            timestamp: get_utc8_timestamp_millis(),
                            request_id: request_id,
                        };
                        register_param
                    }
                    None => {
                        return Err("模拟模式未配置".to_string());
                    }
                }
            }
        };

        self.request_id = Some(request_id);
        tracing::warn!("{}: 注册交易所的请求id: {:?}", self.base_context.node_name, self.request_id);

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
        let params = match self.base_context.trade_mode {
            TradeMode::Live => {
                match self.live_config.clone() {
                    Some(config) => {
                        let params = SubscribeKlineStreamParams {
                            strategy_id: self.base_context.strategy_id.clone(),
                            node_id: self.base_context.node_id.clone(),
                            account_id: config.selected_live_account.account_id,
                            exchange: config.selected_live_account.exchange,
                            symbol: config.symbol,
                            interval: config.interval,
                            frequency: self.frequency.clone(),
                            sender: self.base_context.node_id.clone(),
                            timestamp: get_utc8_timestamp_millis(),
                            request_id: request_id,
                        };
                        params

                    }
                    None => {
                        return Err("实盘模式未配置".to_string());
                    }
                    
                }    
            },
            _ => {
                return Err("不支持的订阅模式".to_string());
            }
        };

        self.request_id = Some(request_id);

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
        let params = UnsubscribeKlineStreamParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: self.account_id.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            interval: self.interval.clone(),
            frequency: self.frequency.clone(),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        // 设置请求id
        self.request_id = Some(request_id);

        let command_event = CommandEvent::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(params));
        if let Err(_) = self.base_context.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_context.node_id,
                "数据源节点发送数据失败"
            );
        }   
        Ok(())
    }

}