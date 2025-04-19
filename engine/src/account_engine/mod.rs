pub mod account_engine_context;
pub mod account_engine_types;

use std::sync::Arc;
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use crate::{exchange_engine::ExchangeEngine, account_engine::account_engine_context::AccountEngineContext};
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use tokio::sync::Mutex;
use std::any::Any;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
// use crate::account_engine::account_engine_types::ExchangeAccountConfig;
use std::collections::HashMap;
use types::market::Exchange;
use types::account::ExchangeAccountConfig;



#[derive(Debug, Clone)]
pub struct AccountEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}


#[async_trait]
impl Engine for AccountEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>> {
        self.context.clone()
    }

    async fn start(&self) {

        let engine_name = self.get_engine_name().await;
        tracing::info!("{}已启动", engine_name);
        self.listen_events().await;
        let mut context = self.context.write().await;
        let account_engine_context = context.as_any_mut().downcast_mut::<AccountEngineContext>().unwrap();
        account_engine_context.monitor_account().await;
    }


}




impl AccountEngine {
    pub fn new(
        event_publisher: EventPublisher,
        account_event_receiver: broadcast::Receiver<Event>,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let context = AccountEngineContext {
            engine_name: EngineName::AccountEngine,
            event_publisher,
            event_receiver: vec![response_event_receiver, request_event_receiver, account_event_receiver],
            exchange_engine,
            database,
            heartbeat,
            accounts: Arc::new(RwLock::new(vec![])),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }

    pub async fn register_mt5_exchange(&self, account_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let account_engine_context = context.as_any_mut().downcast_mut::<AccountEngineContext>().unwrap();
        account_engine_context.register_mt5_exchange(account_id).await
    }
}
