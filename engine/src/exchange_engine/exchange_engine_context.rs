use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use event_center::command::Command;
use event_center::response::exchange_engine_response::{RegisterExchangeResponse};
use event_center::Event;
use event_center::EventPublisher;
use exchange_client::metatrader5::MetaTrader5;
use exchange_client::ExchangeClient;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use types::market::Exchange;
use sea_orm::DatabaseConnection;
use database::query::account_config_query::AccountConfigQuery;
use types::account::AccountConfig;
use event_center::command::exchange_engine_command::UnregisterExchangeParams;
use types::custom_type::AccountId;
use event_center::{EventReceiver, CommandPublisher, CommandReceiver};
use tokio::sync::Mutex;
use types::error::{ExchangeEngineError, ExchangeClientError, error_trait::ErrorContext};
use tracing::instrument;

#[derive(Debug)]
pub struct ExchangeEngineContext {
    pub engine_name: EngineName,
    pub exchanges: HashMap<AccountId, Box<dyn ExchangeClient>>, // 交易所的账户id -> 交易所 每个交易所对应一个账户
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<EventReceiver>,
    pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<CommandReceiver>>,
    pub database: DatabaseConnection,
}

impl Clone for ExchangeEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            exchanges: self
                .exchanges
                .iter()
                .map(|(id, client)| (id.clone(), client.clone_box()))
                .collect(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self
                .event_receiver
                .iter()
                .map(|receiver| receiver.resubscribe())
                .collect(),
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
            database: self.database.clone(),
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

    fn get_event_receiver(&self) -> Vec<EventReceiver> {
        self.event_receiver
            .iter()
            .map(|receiver| receiver.resubscribe())
            .collect()
    }

    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.command_publisher
    }

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.command_receiver.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        let _event = event;

    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::ExchangeEngine(exchange_engine_command) => {
                match exchange_engine_command {
                    ExchangeEngineCommand::RegisterExchange(register_exchange_command) => {
                        let result = self.register_exchange(register_exchange_command.account_id).await;
                        let response = if let Ok(()) = result {
                            // success
                            RegisterExchangeResponse::success("register success", register_exchange_command.account_id, register_exchange_command.exchange)
                        } else {
                            // 注册失败
                            let error = result.unwrap_err();
                            RegisterExchangeResponse::error("register failed", register_exchange_command.account_id, register_exchange_command.exchange, error)
                            
                        };
                        // 发送响应事件
                        register_exchange_command.responder.send(response.into()).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl ExchangeEngineContext {

    pub async fn register_exchange(&mut self, account_id: AccountId) -> Result<(), ExchangeEngineError> {
        // 从数据库中获取账户配置
        let account_config = AccountConfigQuery::get_account_config_by_id(&self.database, account_id).await;
        match account_config {
            Ok(account_config) => {
                match account_config.exchange {
                    Exchange::Metatrader5(_) => {
                        // 判断编译环境，当前逻辑如果是生产环境，则执行下方逻辑
                        // #[cfg(debug_assertions)] - 仅在调试模式下编译（cargo build）
                        // #[cfg(not(debug_assertions))] - 仅在发布模式下编译（cargo build --release）
                        #[cfg(not(debug_assertions))]
                        {
                            // 生产环境
                            Self::register_mt5_exchange(self, account_config).await?;
                        }
                        #[cfg(debug_assertions)]
                        {
                            // 开发环境
                            tracing::debug!("in the dev mode, direct connect to mt5 server");
                            Self::register_mt5_exchange_in_dev(self, account_config).await?;
                        }
                        // Self::register_mt5_exchange(self, account_config).await?;
                        Ok(())

                    }
                    _ => {
                        tracing::error!("不支持的交易所类型: {:?}", account_config.exchange);
                        return Err(ExchangeEngineError::unsupported_exchange_type("unsupported exchange type", account_config.exchange, account_id));
                    }
                }
            }
            Err(e) => {
                return Err(ExchangeEngineError::Database {
                    message: e.to_string(),
                    source: e,
                });
            }
        }
    }


    #[instrument(skip(self, account_config), fields(login = %account_config.config["login"], server = %account_config.config["server"]))]
    async fn register_mt5_exchange_in_dev(&mut self, account_config: AccountConfig) -> Result<(), ExchangeEngineError> {

        let login = account_config.config["login"].as_i64().unwrap();
        let password = account_config.config["password"].as_str().unwrap().to_string();
        let server = account_config.config["server"].as_str().unwrap().to_string();
        let terminal_path = account_config.config["terminal_path"].as_str().unwrap().to_string();
        
        let mut mt5 = MetaTrader5::new(
            account_config.id,
            login,
            password,
            server.clone(),
            terminal_path,
            self.get_event_publisher().clone(),
        );
        
            match mt5.connect_to_server(8001).await.with_context(|| "connect to server".to_string()) {
                Ok(_) => tracing::info!("mt5 server connect success, port: 8001"),                
                Err(e) => {
                    tracing::error!("context1: {}", e);
                    let exchange_client_error = ExchangeClientError::from(e);
                    let error = ExchangeEngineError::register_exchange_failed(
                        "fail to connect to server", 
                        account_config.id, 
                        Exchange::Metatrader5(server.clone()),
                        exchange_client_error);
                    
                    tracing::error!("{}", error.to_string());
                    
                }
            }
        

        // 初始化终端 (带重试机制)
        match mt5.initialize_terminal().await.with_context(|| "initialize terminal".to_string()) {
            Ok(_) => tracing::info!(account_id = %account_config.id, "mt5 terminal is initialized successfully"),
            Err(e) => {
                tracing::error!("context2: {}", e);
                let exchange_client_error = ExchangeClientError::from(e);
                let error = ExchangeEngineError::register_exchange_failed(
                    "fail to initialize terminal", 
                    account_config.id, 
                    Exchange::Metatrader5(server.clone()),
                    exchange_client_error);

                tracing::error!("{}", error.to_string());
            }
        }

        match mt5.connect_websocket().await.with_context(|| "connect to websocket".to_string()) {
            Ok(_) => tracing::info!("MT5-{} websocket connect success", account_config.id),
            Err(e) => {
                tracing::error!("context3: {}", e);
                let exchange_client_error = ExchangeClientError::from(e);
                let error = ExchangeEngineError::register_exchange_failed(
                    "fail to connect to websocket", 
                    account_config.id, 
                    Exchange::Metatrader5(server.clone()),
                    exchange_client_error);
                tracing::error!("{}", error.to_string());
            }
        }
        

        // 存储交易所客户端
        let mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;

        tracing::info!("MT5-{} exchange register success!", account_config.id);
        self.exchanges.insert(account_config.id, mt5_exchange);
        Ok(())
    }


    
    async fn register_mt5_exchange(&mut self, account_config: AccountConfig) -> Result<(), String> {
        
        let mut mt5 = MetaTrader5::new(
            account_config.id,
            account_config.config["login"].as_i64().unwrap(),
            account_config.config["password"].as_str().unwrap().to_string(),
            account_config.config["server"].as_str().unwrap().to_string(),
            account_config.config["terminal_path"].as_str().unwrap().to_string(),
            self.get_event_publisher().clone(),
        );
        
        // 启动mt5服务器 (带重试机制)
        let max_server_retries = 3;
        let mut server_retry_count = 0;
        let mut server_port: Option<u16> = None;
        
        tracing::debug!("开始启动mt5_server");
        while server_retry_count < max_server_retries {
            match tokio::time::timeout(tokio::time::Duration::from_secs(30), mt5.start_mt5_server(false)).await
            {
                Ok(port_result) => {
                    match port_result {
                        Ok(port) => {
                            tracing::info!("MT5-{} 服务器启动成功, 端口: {}", account_config.id, port);
                            server_port = Some(port);
                            break;
                        }
                        Err(_) => {
                            server_retry_count += 1;
                            tracing::error!("MT5-{} 服务器启动失败 (尝试 {}/{})", 
                                account_config.id, server_retry_count, max_server_retries);
                            if server_retry_count >= max_server_retries {
                                return Err(format!("MT5-{} 服务器启动失败，已重试{}次",
                                    account_config.id, max_server_retries));
                            }
                            // 等待一段时间后重试
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        }
                    }
                }
                Err(_) => {
                    server_retry_count += 1;
                    // 超时
                    let error_msg = format!("MT5-{} 服务启动超时 (尝试 {}/{})", 
                        account_config.id, server_retry_count, max_server_retries);
                    tracing::error!("{}", error_msg);
                    if server_retry_count >= max_server_retries {
                        return Err(format!("MT5-{} 服务启动超时，已重试{}次",
                            account_config.id, max_server_retries));
                    }
                    // 等待一段时间后重试
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
        
        if server_port.is_none() {
            return Err(format!("MT5-{} 服务器启动失败，所有重试均失败", account_config.id));
        }

        // 初始化终端 (带重试机制)
        let max_init_retries = 3;
        let mut init_retry_count = 0;
        tracing::debug!("开始初始化终端");
        while init_retry_count < max_init_retries {
            match tokio::time::timeout(tokio::time::Duration::from_secs(30), mt5.initialize_terminal()).await
            {
                Ok(init_result) => {
                    match init_result {
                        Ok(_) => {
                            tracing::info!("MT5-{} 终端初始化成功", account_config.id);
                            break;
                        }
                        Err(_) => {
                            init_retry_count += 1;
                            tracing::error!("MT5-{} 终端初始化失败 (尝试 {}/{})", 
                                account_config.id, init_retry_count, max_init_retries);
                            if init_retry_count >= max_init_retries {
                                return Err(format!("MT5-{} 终端初始化失败，已重试{}次", 
                                    account_config.id, max_init_retries));
                            }
                            // 等待一段时间后重试
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        }
                    }
                }
                Err(_) => {
                    init_retry_count += 1;
                    tracing::error!("MT5-{} 终端初始化超时 (尝试 {}/{})", 
                        account_config.id, init_retry_count, max_init_retries);
                    if init_retry_count >= max_init_retries {
                        return Err(format!("MT5-{} 终端初始化超时，已重试{}次", 
                            account_config.id, max_init_retries));
                    }
                    // 等待一段时间后重试
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        // 连接websocket (带重试机制)
        let max_ws_retries = 3;
        let mut ws_retry_count = 0;
        tracing::debug!("开始连接websocket");
        while ws_retry_count < max_ws_retries {
            match mt5.connect_websocket().await {
                Ok(_) => {
                    tracing::info!("MT5-{} WebSocket连接成功", account_config.id);
                    break;
                }
                Err(_) => {
                    ws_retry_count += 1;
                    tracing::error!("MT5-{} WebSocket连接失败 (尝试 {}/{})", 
                        account_config.id, ws_retry_count, max_ws_retries);
                    if ws_retry_count >= max_ws_retries {
                        return Err(format!("MT5-{} WebSocket连接失败，已重试{}次", 
                            account_config.id, max_ws_retries));
                    }
                    // 等待一段时间后重试
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        // 存储交易所客户端
        let mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;

        tracing::info!("MT5-{} 交易所注册成功!", account_config.id);
        self.exchanges
            .insert(account_config.id, mt5_exchange);
        Ok(())
    }


    pub async fn unregister_exchange(
        &mut self,
        unregister_params: UnregisterExchangeParams,
    ) -> Result<(), String> {
        tracing::debug!("接收到命令: {:?}", unregister_params);
        // 先获取实例
        let mut exchange = self.get_exchange(&unregister_params.account_id).await?;
        match exchange.exchange_type() {
            Exchange::Metatrader5(_) => {
                // 停止mt5服务器，添加超时处理
                let mt5 = exchange.as_any_mut().downcast_mut::<MetaTrader5>().unwrap();
                
                // 设置超时时间为15秒
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(15), 
                    mt5.stop_mt5_server()
                ).await {
                    // 在超时时间内完成了操作
                    Ok(result) => match result {
                        // 停止成功
                        Ok(true) => {
                            tracing::info!("成功停止MT5服务，账户ID: {}", unregister_params.account_id);
                            self.exchanges.remove(&unregister_params.account_id);
                        },
                        // 停止尝试但失败
                        Ok(false) => {
                            tracing::error!("MT5服务停止失败，但仍将移除实例，账户ID: {}", unregister_params.account_id);
                            self.exchanges.remove(&unregister_params.account_id);
                        },
                        // 函数执行出错
                        Err(e) => {
                            tracing::error!("MT5服务停止出错，错误: {}，账户ID: {}", e, unregister_params.account_id);
                            self.exchanges.remove(&unregister_params.account_id);
                        }
                    },
                    // 操作超时
                    Err(_) => {
                        tracing::error!("MT5服务停止操作超时，账户ID: {}", unregister_params.account_id);
                        // 尽管超时，仍然移除实例，避免资源泄漏
                        self.exchanges.remove(&unregister_params.account_id);
                    }
                }
            }
            _ => {
                // 对于其他类型的交易所，直接移除
                self.exchanges.remove(&unregister_params.account_id);
            }
        }

        Ok(())
    }

    pub async fn is_registered(&self, account_id: &i32) -> bool {
        self.exchanges.contains_key(account_id)
    }

    pub async fn get_exchange(&self, account_id: &i32) -> Result<Box<dyn ExchangeClient>, String> {
        match self.exchanges.get(account_id) {
            Some(client) => {
                // 使用clone_box方法直接获取一个新的Box<dyn ExchangeClient>
                Ok(client.clone_box())
            }
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }

    pub async fn get_exchange_ref<'a>(
        &'a self,
        account_id: &i32,
    ) -> Result<&'a Box<dyn ExchangeClient>, String> {
        match self.exchanges.get(account_id) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }

    // 添加一个获取可变引用的方法
    pub async fn get_exchange_mut<'a>(
        &'a mut self,
        account_id: &i32,
    ) -> Result<&'a mut Box<dyn ExchangeClient>, String> {
        match self.exchanges.get_mut(account_id) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }
}
