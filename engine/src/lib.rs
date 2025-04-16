pub mod engine_manager; // 引擎管理器
pub mod market_engine; // 市场引擎
pub mod exchange_engine; // 交易所引擎
pub mod indicator_engine; // 指标引擎
pub mod order_engine; // 订单引擎
pub mod strategy_engine; // 策略引擎
pub mod cache_engine; // 缓存引擎
pub mod position_engine; // 持仓引擎
pub mod transaction_engine; // 交易明细引擎
pub mod account_engine; // 账户引擎



use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use event_center::Event;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
use strum::{EnumString, Display};
use serde::{Deserialize, Serialize};
use event_center::EventPublisher;



#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
pub enum EngineName {
    #[strum(serialize="exchange-engine")]
    ExchangeEngine, // 交易所引擎
    #[strum(serialize="marekt-engine")]
    MarketEngine, // 市场引擎
    #[strum(serialize="indicator-engine")]
    IndicatorEngine, // 指标引擎
    #[strum(serialize="order-engine")]
    OrderEngine, // 订单引擎
    #[strum(serialize="strategy-engine")]
    StrategyEngine, // 策略引擎
    #[strum(serialize="cache-engine")]
    CacheEngine, // 缓存引擎
    #[strum(serialize="position-engine")]
    PositionEngine, // 持仓引擎
    #[strum(serialize="transaction-engine")]
    TransactionEngine, // 交易明细引擎
    #[strum(serialize="account-engine")]
    AccountEngine, // 账户引擎

}



#[async_trait]
pub trait EngineContext: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn EngineContext>;

    fn get_engine_name(&self) -> EngineName;

    fn get_event_publisher(&self) -> &EventPublisher; 

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>>;

    async fn handle_event(&mut self, event: Event);
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
    async fn listen_events(&self) {
        let context = self.get_context();
        EngineFunction::listen_events(context).await;
    }
    async fn start(&self) {
        let engine_name = self.get_engine_name().await;
        tracing::info!("{}已启动", engine_name);
        self.listen_events().await;


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
}






