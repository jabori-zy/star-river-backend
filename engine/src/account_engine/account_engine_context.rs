use types::market::Exchange;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::EngineContext;
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use std::sync::Arc;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use tokio::sync::RwLock;
use event_center::command::Command;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use event_center::response::exchange_engine_response::ExchangeEngineResponse;
use event_center::account_event::AccountEvent;
use types::account::ExchangeStatus;
use types::account::Account;   
use database::mutation::account_info_mutation::AccountInfoMutation;
use database::query::account_config_query::AccountConfigQuery;
use event_center::command::exchange_engine_command::UnregisterExchangeParams;
use event_center::command::exchange_engine_command::RegisterExchangeParams;
use tokio::sync::oneshot;
use event_center::EventCenterSingleton;

#[derive(Debug)]
pub struct AccountEngineContext {
    pub engine_name: EngineName,
    pub database: DatabaseConnection,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub monitor_account_list: Arc<RwLock<Vec<Account>>>
}

impl Clone for AccountEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            database: self.database.clone(),
            exchange_engine: self.exchange_engine.clone(),
            heartbeat: self.heartbeat.clone(),
            monitor_account_list: self.monitor_account_list.clone(),
        }
    }
}

#[async_trait]
impl EngineContext for AccountEngineContext {
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

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Account(account_event) => {
                match account_event {
                    AccountEvent::AccountConfigAdded(account_id) => {
                        // 将账户配置添加到accounts中
                        self.update_monitor_accounts(account_id).await;
                    }
                    AccountEvent::AccountConfigDeleted(account_id) => {
                        // 将账户配置从accounts中删除
                        self.delete_monitor_accounts(account_id).await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }

    async fn handle_command(&mut self, command: Command) {
        let _command = command;
    }
}


impl AccountEngineContext {

    async fn delete_monitor_accounts(&mut self, account_id: i32) {
        tracing::debug!("删除账户: {:?}", account_id);
        let accounts = {
            let accounts = self.monitor_account_list.read().await;
            accounts.clone()
        };
        let index = accounts.iter().position(|account| account.get_account_id() == account_id).unwrap();
        let mut accounts = self.monitor_account_list.write().await;
        accounts.remove(index);
        // 同时向exchange_engine发送注销交易所的命令
        let (resp_tx, resp_rx) = oneshot::channel();
        let unregister_params = UnregisterExchangeParams {
            account_id: account_id,
            sender: "account_engine".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            responder: resp_tx,
        };
        let command_event = ExchangeEngineCommand::UnregisterExchange(unregister_params);
        // self.get_command_publisher().send(command_event.into()).await.unwrap();
        EventCenterSingleton::send_command(command_event.into()).await.unwrap();

    }

    // 更新监控账户
    async fn update_monitor_accounts(&mut self, account_id: i32) {
        tracing::debug!("更新账户: {:?}", account_id);
        let new_account = AccountConfigQuery::get_account_config_by_id(&self.database, account_id).await.unwrap();
        tracing::debug!("获取到的账户配置: {:?}", new_account);
        let account = Account::new(new_account, None, ExchangeStatus::NotRegist);
        self.monitor_account_list.write().await.push(account);
        
    }

    // 监控账户的状态
    pub async fn monitor_accounts(&mut self) {
        
        // 获取所有的账户配置
        {
            let all_account_config = AccountConfigQuery::get_all_account_config(&self.database).await.unwrap();
            let account = all_account_config.iter().map(|account_config| Account::new(account_config.clone(), None, ExchangeStatus::NotRegist)).collect::<Vec<Account>>();
            // tracing::debug!("监控账户的交易所状态: {:?}", account);
            self.monitor_account_list.write().await.extend(account.into_iter());
        }

        let accounts = self.monitor_account_list.clone();
        let exchange_engine = self.exchange_engine.clone();
        // let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控账户的交易所状态".to_string(),
            move || {
                let accounts = accounts.clone();
                let exchange_engine = exchange_engine.clone();
                // let event_publisher = event_publisher.clone();
                async move {
                    Self::process_exchange_status(
                        accounts,
                        exchange_engine,
                        // event_publisher,
                    ).await
                }
            },
            10
        ).await;

        let accounts = self.monitor_account_list.clone();
        let exchange_engine = self.exchange_engine.clone();
        // let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        heartbeat.register_async_task(
            "监控账户信息".to_string(),
            move || {
                let accounts = accounts.clone();
                let exchange_engine = exchange_engine.clone();
                // let event_publisher = event_publisher.clone();
                let database = database.clone();
                async move {
                    Self::process_account_info(
                        accounts,
                        exchange_engine,
                        // event_publisher,
                        database,
                    ).await
                }
            },
            10
        ).await;


    }

    // 监控账户的交易所状态
    async fn process_exchange_status(
        accounts: Arc<RwLock<Vec<Account>>>, 
        exchange_engine: Arc<Mutex<ExchangeEngine>>, 
        // event_publisher: EventPublisher
    ) {
        
        let accounts_clone = {
            let accounts = accounts.read().await;
            accounts.clone()
        };

        // 如果vec为空，则直接返回
        if accounts_clone.is_empty() {
            return;
        }

        // 遍历账户配置,尝试获取账户信息
        for (index, account) in accounts_clone.iter().enumerate() {
            // 先判断账户的交易所状态
            let account_status = account.get_exchange_status();
            match account_status {
                // 未注册
                ExchangeStatus::NotRegist => {
                    continue;
                },
                // 注册中
                ExchangeStatus::Registing => {
                    // 如果注册中，跳过
                    tracing::debug!("{}账户正在注册中，跳过", account.get_account_name());
                    continue;
                },
                // 已注册
                ExchangeStatus::Registed => {
                    // 通过exchange_engine二次确认
                    
                    let exchange_engine_guard = exchange_engine.lock().await;
                    let is_registered = exchange_engine_guard.is_registered(&account.get_account_id()).await;
                    // 如果exchange_engine未注册，则将状态设置为未注册
                    if !is_registered {
                        let mut accounts = accounts.write().await;
                        accounts[index].set_exchange_status(ExchangeStatus::NotRegist);
                    }

                },
                // 注册失败
                _ => {}
            }
        }
    }


    async fn process_account_info(
        accounts: Arc<RwLock<Vec<Account>>>, 
        exchange_engine: Arc<Mutex<ExchangeEngine>>, 
        // event_publisher: EventPublisher,
        database: DatabaseConnection
    ) {
        let accounts_clone = {
            let accounts = accounts.read().await;
            accounts.clone()
        };

        // 如果vec为空，则直接返回
        if accounts_clone.is_empty() {
            return;
        }

        // 遍历账户配置,尝试获取账户信息
        for (index, account) in accounts_clone.iter().enumerate() {
            // tracing::debug!("监控账户信息: {:?}", account);
            // 先判断账户的交易所的注册状态
            let account_status = account.get_exchange_status();
            match account_status {
                ExchangeStatus::Registed => {
                    // 获取账户信息
                    let exchange = exchange_engine.lock().await;
                    let exchange = exchange.get_exchange(&account.get_account_id()).await;
                    match exchange {
                        Ok(exchange) => {
                            // 如果获取账户信息错误，则将状态设置为注册失败
                            let account_info = exchange.get_account_info().await;
                            match account_info {    
                                Ok(account_info) => {
                                    // 1.更新数据库
                                    let account_info = AccountInfoMutation::update_account_info(&database, account.get_account_id(), account_info.to_json()).await.unwrap();
                                    // 2.更新账户信息
                                    let mut accounts = accounts.write().await;
                                    accounts[index].set_account_info(account_info);
                                    // 3.发布账户已更新事件
                                    let account_updated_event = AccountEvent::AccountUpdated(account.clone());
                                    // event_publisher.publish(account_updated_event.into()).await.unwrap();
                                    EventCenterSingleton::publish(account_updated_event.into()).await.unwrap();
                                }
                                Err(e) => {
                                    let mut accounts = accounts.write().await;
                                    accounts[index].set_exchange_status(ExchangeStatus::NotRegist);
                                    // tracing::error!("获取账户信息失败: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            // let mut accounts = accounts.write().await;
                            // accounts[index].set_exchange_status(ExchangeStatus::Error);
                            // tracing::error!("获取账户信息失败: {:?}", e);
                        }
                    }
                    
                            
                            // 如果获取账户信息错误，则将状态设置为注册失败
                    // 如果获取账户信息错误，则将状态设置为注册失败
                    
                }
                ExchangeStatus::NotRegist => {
                    let exchange = account.get_exchange();
                    match exchange {
                        Exchange::Metatrader5(_) => {
                            // 发布账户已更新事件
                            let account_updated_event = AccountEvent::AccountUpdated(account.clone());
                            // event_publisher.publish(account_updated_event.into()).await.unwrap();
                            EventCenterSingleton::publish(account_updated_event.into()).await.unwrap();
                        }
                        _ => {}
                    }
                }
                _ => continue
            }
        }

    }


    pub async fn register_exchange(&mut self, account_id: i32) -> Result<(), String> {
        // 获取account_id的config
        let (resp_tx, resp_rx) = oneshot::channel();
        let account_config = AccountConfigQuery::get_account_config_by_id(&self.database, account_id).await.unwrap();
        let register_params = RegisterExchangeParams {
            account_id: account_config.id,
            exchange: account_config.exchange,
            sender: "account_engine".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            responder: resp_tx,
        };
        let register_exchange_command = ExchangeEngineCommand::RegisterExchange(register_params);
        // self.get_command_publisher().send(register_exchange_command.into()).await.unwrap();
        EventCenterSingleton::send_command(register_exchange_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            let exchange_engine_response = ExchangeEngineResponse::try_from(response);
            if let Ok(exchange_engine_response) = exchange_engine_response {
                match exchange_engine_response {
                    ExchangeEngineResponse::RegisterExchange(register_exchange_response) => {
                        tracing::info!("账户引擎收到注册交易所响应: {:?}", register_exchange_response);
                        let mut accounts = self.monitor_account_list.write().await;
                        let index = accounts.iter().position(|account| account.get_account_id() == register_exchange_response.account_id).unwrap();
                        accounts[index].set_exchange_status(ExchangeStatus::Registed);
                    }
                }
            }
        }
        Ok(())
    }

    
}


