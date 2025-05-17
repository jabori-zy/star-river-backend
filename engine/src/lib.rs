pub mod engine_manager; // 引擎管理器
pub mod market_engine; // 市场引擎
pub mod exchange_engine; // 交易所引擎
pub mod indicator_engine; // 指标引擎

pub mod strategy_engine; // 策略引擎
// pub mod cache_engine; // 缓存引擎
pub mod account_engine; // 账户引擎
pub mod cache_engine;



use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use types::custom_type::StrategyId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use event_center::Event;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
use strum::{EnumString, Display};
use serde::{Deserialize, Serialize};
use event_center::{EventPublisher, CommandPublisher};
use types::engine::EngineName;
use event_center::CommandReceiver;
use event_center::command::Command;


#[async_trait]
pub trait EngineContext: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn EngineContext>;

    fn get_engine_name(&self) -> EngineName;

    fn get_event_publisher(&self) -> &EventPublisher; 

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>>;

    fn get_command_publisher(&self) -> &CommandPublisher;

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>>;

    async fn handle_event(&mut self, event: Event);

    async fn handle_command(&mut self, command: Command);
}

impl Clone for Box<dyn EngineContext> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}




#[async_trait]
pub trait Engine : Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn Engine>;

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>>;
    
    async fn get_engine_name(&self) -> EngineName {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_engine_name()
    }

    // 监听事件
    async fn listen_events(&self) {
        let context = self.get_context();
        EngineFunction::listen_events(context).await;
    }

    async fn listen_commands(&self) {
        let context = self.get_context();
        EngineFunction::listen_commands(context).await;
    }


    async fn start(&self) {
        let engine_name = self.get_engine_name().await;
        tracing::info!("{}已启动", engine_name);
        // 监听事件
        self.listen_events().await;
        // 监听命令
        self.listen_commands().await;
    }
}


pub struct EngineFunction;

impl EngineFunction {
    pub async fn listen_events(context: Arc<RwLock<Box<dyn EngineContext>>>) {

        let (engine_name, event_receivers) = {
            let context_guard = context.read().await;
            let engine_name = context_guard.get_engine_name();
            let event_receivers : Vec<broadcast::Receiver<Event>>= context.read().await.get_event_receiver()
            .iter()
            .map(|r| r.resubscribe())
            .collect();
            (engine_name, event_receivers)

        };

        if event_receivers.is_empty() {
            tracing::warn!("{}: 没有事件接收器", engine_name.clone());
            return;
        }

        let streams: Vec<_> = event_receivers.into_iter()
            .map(|receiver| BroadcastStream::new(receiver))
            .collect();

        let mut combined_stream = select_all(streams);

        tracing::debug!("{}: 开始监听事件", engine_name);
        tokio::spawn(async move {
            loop {
                if let Some(received_event) = combined_stream.next().await {
                    match received_event {
                        Ok(event) => {
                            let mut context_guard = context.write().await;
                                // tracing::debug!("{}: 接收到事件: {:?}", engine_name, event);
                                context_guard.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::error!("节点{}接收事件错误: {}", engine_name, e);
                        }
                        
                    }
                }
            }

        });


    }

    pub async fn listen_commands(context: Arc<RwLock<Box<dyn EngineContext>>>) {
        let (engine_name, command_receiver) = {
            let context_guard = context.read().await;
            let engine_name = context_guard.get_engine_name();
            let command_receiver = context.read().await.get_command_receiver();
            (engine_name, command_receiver)

        };
        tracing::debug!("{}: 开始监听命令", engine_name);
        tokio::spawn(async move {
            loop {
                if let Some(received_command) = command_receiver.lock().await.recv().await {
                    let mut context_guard = context.write().await;
                    // tracing::debug!("{}: 接收到事件: {:?}", engine_name, event);
                    context_guard.handle_command(received_command).await;
                    }
                }
            }
        );
    }
}






