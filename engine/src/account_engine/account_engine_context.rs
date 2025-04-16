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
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use event_center::order_event::OrderEvent;
use types::order::Order;
use event_center::command_event::position_engine_command::GetPositionParam;
use types::position::Position;
use database::mutation::position_mutation::PositionMutation;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::account_engine::account_engine_types::AccountConfig;
use database::mutation::account_config_mutation::AccountConfigMutation;


#[derive(Debug)]
pub struct AccountEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub database: DatabaseConnection,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub account_info: Arc<RwLock<HashMap<Exchange, AccountConfig>>>
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
            account_info: self.account_info.clone(),
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
    }
}


impl AccountEngineContext {
    // 添加账户配置
    pub async fn add_account_config(&mut self, account_name: String, account_config: AccountConfig) -> Result<(), String> {
        match account_config {
            AccountConfig::MetaTrader5(meta_trader5_account_config) => {
                let exchange = Exchange::Metatrader5;
                let account_config_json = serde_json::to_value(meta_trader5_account_config).expect("Failed to serialize meta trader 5 account config");
                let account_config = AccountConfigMutation::insert_account_config(
                    &self.database, 
                    account_name,
                    exchange,
                    account_config_json
                ).await.expect("Failed to insert account config");
                tracing::info!("Account config added: {:?}", account_config);
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn load_account_config(&mut self) {
        let account_config = AccountConfigMutation::get_all_account_config(&self.database).await.expect("Failed to get account config");
        tracing::info!("Account config: {:?}", account_config);
    }

    // 监控账户
    async fn monitor_account(&mut self) {
        let account_config = self.account_info.clone();
        let exchange_engine = self.exchange_engine.clone();
        let event_publisher = self.event_publisher.clone();
        let database = self.database.clone();
        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控账户".to_string(),
            move || {
                let account_config = account_config.clone();
                let exchange_engine = exchange_engine.clone();
                let event_publisher = event_publisher.clone();
                let database = database.clone();
                async move {
                    Self::process_account(
                        account_config,
                        exchange_engine,
                        event_publisher,
                        database
                    ).await
                }
            },
            10
        ).await;
    }

    async fn process_account(
        account_config: Arc<RwLock<HashMap<Exchange, AccountConfig>>>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        event_publisher: EventPublisher,
        database: DatabaseConnection,
    ) {
        let account_config_clone = {
            let account_config = account_config.read().await;
            account_config.clone()
        };

        // 如果hashmap为空，则直接返回
        if account_config_clone.is_empty() {
            return;
        }

        // 遍历账户配置
        for (exchange, account_config) in account_config_clone.iter() {
            match exchange {
                Exchange::Metatrader5 => {
                    // 获取交易所的上下文
                    let exchange_engine_guard = exchange_engine.lock().await;
                    // 获取交易所对象
                    let exchange = exchange_engine_guard.get_exchange(&exchange).await;
                    // 获取账户信息
                    let account_info = exchange.get_account_info().await;
                    tracing::info!("Account info: {:?}", account_info);
                }
                _ => {}
            }
        }
    }

    
}


