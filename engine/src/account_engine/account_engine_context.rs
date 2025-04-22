use tokio::sync::broadcast;
use types::market::Exchange;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use std::sync::Arc;
use event_center::EventPublisher;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::collections::HashMap;
use tokio::sync::RwLock;
// use crate::account_engine::account_engine_types::ExchangeAccountConfig;
use database::mutation::mt5_account_config_mutation::Mt5AccountConfigMutation;
use database::query::mt5_account_config_query::Mt5AccountConfigQuery;
use types::account::mt5_account::Mt5AccountConfig;
use types::account::ExchangeAccountConfig;
use event_center::command_event::exchange_engine_command::RegisterMt5ExchangeParams;
use event_center::command_event::exchange_engine_command::UnregisterMt5ExchangeParams;
use uuid::Uuid;
use event_center::command_event::CommandEvent;
use event_center::command_event::exchange_engine_command::ExchangeEngineCommand;
use event_center::response_event::ResponseEvent;
use event_center::response_event::exchange_engine_response::ExchangeEngineResponse;
use event_center::account_event::AccountEvent;
use types::account::mt5_account::Mt5Account;
use types::account::ExchangeStatus;
use types::account::{AccountTrait,Account};   
use database::mutation::mt5_account_info_mutation::Mt5AccountInfoMutation;
use types::account::mt5_account::Mt5AccountInfo;
use types::account::mt5_account::OriginalMt5AccountInfo;
use types::account::ExchangeAccountInfo;

#[derive(Debug)]
pub struct AccountEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub database: DatabaseConnection,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub accounts: Arc<RwLock<Vec<Box<dyn AccountTrait>>>>
}

impl Clone for AccountEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            database: self.database.clone(),
            exchange_engine: self.exchange_engine.clone(),
            heartbeat: self.heartbeat.clone(),
            accounts: self.accounts.clone(),
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

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            // Event::Response(response_event) => {
            //     self.handle_response_event(response_event).await;
            // }
            Event::Response(response_event) => {
                match response_event {
                    ResponseEvent::ExchangeEngine(exchange_engine_event) => {
                        match exchange_engine_event {
                            ExchangeEngineResponse::RegisterMt5ExchangeSuccess(register_response) => {
                                // self.handle_register_mt5_exchange_success(register_params).await;
                                tracing::debug!("注册mt5交易所成功: {:?}", register_response);
                                // 更新账户的交易所状态
                                let mut accounts = self.accounts.write().await;
                                let index = accounts.iter().position(|account| account.get_account_id() == register_response.terminal_id).unwrap();
                                let account = accounts[index].as_any_mut().downcast_mut::<Mt5Account>().unwrap();
                                account.set_exchange_status(ExchangeStatus::Registed);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::Account(account_event) => {
                match account_event {

                    AccountEvent::AccountConfigAdded((account_id, exchange)) => {
                        // 将账户配置添加到accounts中
                        self.update_accounts(account_id, exchange).await;
                    }
                    AccountEvent::AccountConfigDeleted(account_id) => {
                        // 将账户配置从accounts中删除
                        self.delete_accounts(account_id).await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }
}


impl AccountEngineContext {

    async fn delete_accounts(&mut self, account_id: i32) {
        tracing::debug!("删除账户: {:?}", account_id);
        let accounts = {
            let accounts = self.accounts.read().await;
            accounts.clone()
        };
        let index = accounts.iter().position(|account| account.get_account_id() == account_id).unwrap();
        let mut accounts = self.accounts.write().await;
        accounts.remove(index);
        // 同时向exchange_engine发送注销交易所的命令
        let unregister_params = UnregisterMt5ExchangeParams {
            terminal_id: account_id,
            sender: "account_engine".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::UnregisterMt5Exchange(unregister_params));
        self.event_publisher.publish(command_event.into()).unwrap();

    }

    async fn update_accounts(&mut self, account_id: i32, exchange: Exchange) {
        tracing::debug!("更新账户: {:?}", account_id);
        match exchange {
            Exchange::Metatrader5 => {
                let new_account = Mt5AccountConfigQuery::get_mt5_account_config_by_id(&self.database, account_id).await.unwrap();
                tracing::debug!("获取到的mt5账户配置: {:?}", new_account);
                if new_account.is_some() {
                    let new_account = new_account.unwrap();
                    let account = Mt5Account {
                        account_config: new_account.clone_box().as_any().downcast_ref::<Mt5AccountConfig>().unwrap().clone(),
                        account_info: None,
                        exchange_status: ExchangeStatus::NotRegist,
                    };
                    self.accounts.write().await.push(Box::new(account));
                }
            }
            _ => {}
        }
    }

    // 监控账户的状态
    pub async fn monitor_account(&mut self) {
        
        // 获取所有的账户配置
        {
            let mt5_account_config = Mt5AccountConfigQuery::get_mt5_account_config(&self.database).await.unwrap();
            let account = mt5_account_config.iter().map(|account_config| Mt5Account {
                account_config: account_config.clone_box().as_any().downcast_ref::<Mt5AccountConfig>().unwrap().clone(),
                account_info: None,
                exchange_status: ExchangeStatus::NotRegist, // 默认未注册
            }).collect::<Vec<Mt5Account>>();
            tracing::debug!("监控账户的交易所状态: {:?}", account);
            self.accounts.write().await.extend(account.into_iter().map(|account| Box::new(account) as Box<dyn AccountTrait>));
        }

        let accounts = self.accounts.clone();
        let exchange_engine = self.exchange_engine.clone();
        let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控账户的交易所状态".to_string(),
            move || {
                let accounts = accounts.clone();
                let exchange_engine = exchange_engine.clone();
                let event_publisher = event_publisher.clone();
                async move {
                    Self::process_exchange_status(
                        accounts,
                        exchange_engine,
                        event_publisher,
                    ).await
                }
            },
            10
        ).await;

        let accounts = self.accounts.clone();
        let exchange_engine = self.exchange_engine.clone();
        let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        heartbeat.register_async_task(
            "监控账户信息".to_string(),
            move || {
                let accounts = accounts.clone();
                let exchange_engine = exchange_engine.clone();
                let event_publisher = event_publisher.clone();
                let database = database.clone();
                async move {
                    Self::process_account_info(
                        accounts,
                        exchange_engine,
                        event_publisher,
                        database,
                    ).await
                }
            },
            10
        ).await;


    }

    // 监控账户的交易所状态
    async fn process_exchange_status(accounts: Arc<RwLock<Vec<Box<dyn AccountTrait>>>>, exchange_engine: Arc<Mutex<ExchangeEngine>>, event_publisher: EventPublisher) {
        
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
                    // tracing::debug!("{}账户未注册，开始检查是否已注册", account.get_account_name());
                    // // 从exchange_engine判断是否真的未注册
                    // let exchagne_is_registered = {
                    //     let exchange_engine_guard = exchange_engine.lock().await;
                    //     let is_registered = exchange_engine_guard.is_registered(&account.get_account_id()).await;
                    //     is_registered
                    // };

                    // // 如果是已注册状态，则设置为registed
                    // if exchagne_is_registered {
                    //     let mut accounts = accounts.write().await;
                    //     accounts[index].set_exchange_status(ExchangeStatus::Registed);
                    // }
                    // // 如果是未注册状态，则发送注册命令
                    // else {
                    //     tracing::debug!("{}账户未注册交易所，开始注册", account.get_account_name());
                    //     // 如果没有注册交易所，则发送注册命令
                    //     match account.get_exchange() {
                    //         Exchange::Metatrader5 => {
                    //             // 获取账户配置
                    //             let account_config = account.get_account_config();
                    //             let mt5_account_config = account_config.as_any().downcast_ref::<Mt5AccountConfig>().unwrap();
                    //             let register_params = RegisterMt5ExchangeParams {
                    //                 account_id: mt5_account_config.id,
                    //                 login: mt5_account_config.login,
                    //                 password: mt5_account_config.password.clone(),
                    //                 server: mt5_account_config.server.clone(),
                    //                 terminal_path: mt5_account_config.terminal_path.clone(),
                    //                 sender: "account_engine".to_string(),
                    //                 timestamp: chrono::Utc::now().timestamp_millis(),
                    //                 request_id: Uuid::new_v4(),
                    //             };
                    //             let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterMt5Exchange(register_params));
                    //             event_publisher.publish(command_event.into()).unwrap();
                    //             // 将账户状态设置为注册中
                    //             let mut accounts = accounts.write().await;
                    //             accounts[index].set_exchange_status(ExchangeStatus::Registing);
                    //         }
                    //         _ => {}
                    //     }
                    // }
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
        accounts: Arc<RwLock<Vec<Box<dyn AccountTrait>>>>, 
        exchange_engine: Arc<Mutex<ExchangeEngine>>, 
        event_publisher: EventPublisher,
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
            // 先判断账户的交易所状态
            let account_status = account.get_exchange_status();
            match account_status {
                ExchangeStatus::Registed => {
                    // 获取账户信息
                    let exchange = exchange_engine.lock().await;
                    let exchange = exchange.get_exchange(&account.get_account_id()).await;
                    // 如果获取账户信息错误，则将状态设置为注册失败
                    let account_info = exchange.get_account_info().await;
                    match account_info {    
                        Ok(account_info) => {
                            // tracing::debug!("accounts: {:?}", accounts);
                            // 发布账户信息已更新事件
                            let exchange = account.get_exchange();
                            match exchange {
                                Exchange::Metatrader5 => {

                                    // 1.更新数据库
                                    let mt5_original_account_info = account_info.as_any().downcast_ref::<OriginalMt5AccountInfo>().unwrap().clone();
                                    let account_info = Mt5AccountInfoMutation::update_mt5_account_info(&database, mt5_original_account_info).await.unwrap();
                                    // 2.更新账户信息
                                    let mut accounts = accounts.write().await;
                                    accounts[index].set_account_info(account_info.clone_box());
                                    // 3.发布账户已更新事件
                                    let mt5_account = account.as_any().downcast_ref::<Mt5Account>().unwrap().clone();
                                    let account_updated_event = AccountEvent::AccountUpdated(Account::Mt5Account(mt5_account));
                                    event_publisher.publish(account_updated_event.into()).unwrap();

                                    
                                }
                                _ => {}
                            }
                            
                            
                        }
                        Err(e) => {
                            let mut accounts = accounts.write().await;
                            accounts[index].set_exchange_status(ExchangeStatus::Error);
                            tracing::error!("获取账户信息失败: {:?}", e);
                        }
                    }
                }
                ExchangeStatus::NotRegist => {
                    let exchange = account.get_exchange();
                    match exchange {
                        Exchange::Metatrader5 => {
                            // 发布账户已更新事件
                            let mt5_account = account.as_any().downcast_ref::<Mt5Account>().unwrap().clone();
                            let account_updated_event = AccountEvent::AccountUpdated(Account::Mt5Account(mt5_account));
                            event_publisher.publish(account_updated_event.into()).unwrap();
                        }
                        _ => {}
                    }
                }
                _ => continue
            }
        }

    }


    pub async fn register_mt5_exchange(&mut self, account_id: i32) -> Result<(), String> {
        // 获取account_id的config

        let mt5_account_config = Mt5AccountConfigQuery::get_mt5_account_config_by_id(&self.database, account_id).await.unwrap();
        if mt5_account_config.is_none() {
            return Err("账户配置不存在".to_string());
        }
        let mt5_account_config = mt5_account_config.unwrap();
        let register_params = RegisterMt5ExchangeParams {
            account_id: mt5_account_config.id,
            login: mt5_account_config.login,
            password: mt5_account_config.password,
            server: mt5_account_config.server,
            terminal_path: mt5_account_config.terminal_path,
            sender: "account_engine".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterMt5Exchange(register_params));
        self.event_publisher.publish(command_event.into()).unwrap();
        Ok(())
    }

    
}


