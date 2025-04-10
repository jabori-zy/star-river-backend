use tokio::sync::broadcast;
use types::market::Exchange;
use event_center::Event;
use crate::EngineContext;
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use event_center::command_event::CommandEvent;
use event_center::command_event::exchange_engine_command::{RegisterExchangeParams, ExchangeEngineCommand};
use event_center::response_event::ResponseEvent;
use event_center::response_event::exchange_engine_response::{ExchangeEngineResponse, RegisterExchangeSuccessResponse};
use event_center::EventPublisher;
use exchange_client::ExchangeClient;
use std::collections::HashMap;
use utils::get_utc8_timestamp;
use exchange_client::binance::BinanceExchange;
use exchange_client::metatrader5::MetaTrader5;



#[derive(Debug)]
pub struct ExchangeEngineContext {
    pub engine_name: EngineName,
    pub exchanges: HashMap<Exchange, Box<dyn ExchangeClient>>,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
}

impl Clone for ExchangeEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            exchanges: self.exchanges.iter().map(|(exchange, client)| (exchange.clone(), client.clone_box())).collect(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
        }
    }
}

#[async_trait]
impl EngineContext for ExchangeEngineContext {

    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Command(command_event) => {
                match command_event {
                    CommandEvent::ExchangeEngine(exchange_manager_command) => {
                        match exchange_manager_command {
                            ExchangeEngineCommand::RegisterExchange(register_exchange_command) => {
                                tracing::debug!("接收到命令: {:?}", register_exchange_command);
                                self.register_exchange(register_exchange_command).await.expect("注册交易所失败");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }

}


impl ExchangeEngineContext {
    async fn register_exchange(&mut self, register_params: RegisterExchangeParams) -> Result<(), String>{
        // 检查是否已经注册
        let should_register = {
            !self.exchanges.contains_key(&register_params.exchange)
        };

        if !should_register {
            // 直接发送响应事件
            tracing::warn!("{}交易所已注册, 无需重复注册", register_params.exchange);
            let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                exchange: register_params.exchange.clone(),
                response_timestamp: get_utc8_timestamp(),
                response_id: register_params.request_id,
            }));
            self.get_event_publisher().publish(response_event.clone().into()).unwrap();
            return Ok(());
        }
        
        match register_params.exchange {
            Exchange::Binance => {
                // 当类型为Box<dyn Trait Bound>时，需要显式地指定类型
                
                let mut binance_exchange = Box::new(BinanceExchange::new(self.get_event_publisher().clone())) as Box<dyn ExchangeClient>;
                binance_exchange.connect_websocket().await?;
                
                tracing::info!("{}交易所注册成功!", register_params.exchange);
                self.exchanges.insert(register_params.exchange.clone(), binance_exchange);
                // 发送响应事件

                let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                    exchange: register_params.exchange.clone(),
                    response_timestamp: get_utc8_timestamp(),
                    response_id: register_params.request_id,
                }));
                self.get_event_publisher().publish(response_event.clone().into()).unwrap();
                
                Ok(())

            }
            Exchange::Metatrader5 => {
                let mut mt5 = MetaTrader5::new(self.get_event_publisher().clone());
                // 启动mt5服务器
                
                mt5.start_mt5_server(false).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                mt5.initialize_client().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // mt5.login(23643, "HhazJ520!!!!", "EBCFinancialGroupKY-Demo", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
                mt5.login(76898751, "HhazJ520....", "Exness-MT5Trial5", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let mut mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;
                mt5_exchange.connect_websocket().await?;
                tracing::info!("{}交易所注册成功!", register_params.exchange);

                self.exchanges.insert(register_params.exchange.clone(), mt5_exchange);
                let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                    exchange: register_params.exchange.clone(),
                    response_timestamp: get_utc8_timestamp(),
                    response_id: register_params.request_id,
                }));
                self.get_event_publisher().publish(response_event.clone().into()).unwrap();
                Ok(())
            }
            

            _ => {
                return Err("不支持的交易所".to_string());
            }
        }
    }

    pub async fn is_registered(&self, exchange: &Exchange) -> bool {
        self.exchanges.contains_key(exchange)
    }

    pub async fn get_exchange(&self, exchange: &Exchange) -> Result<Box<dyn ExchangeClient>, String> {
        
        match self.exchanges.get(exchange) {
            Some(client) => {
                // 使用clone_box方法直接获取一个新的Box<dyn ExchangeClient>
                Ok(client.clone_box())
            },
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }

    pub async fn get_exchange_ref<'a>(&'a self, exchange: &Exchange) -> Result<&'a Box<dyn ExchangeClient>, String> {
        match self.exchanges.get(exchange) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }

    // 添加一个获取可变引用的方法
    pub async fn get_exchange_mut<'a>(&'a mut self, exchange: &Exchange) -> Result<&'a mut Box<dyn ExchangeClient>, String> {
        match self.exchanges.get_mut(exchange) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }



}