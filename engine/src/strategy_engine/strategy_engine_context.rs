use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use sea_orm::DatabaseConnection;
use database::entities::strategy_info::Model as StrategyInfo;
use database::query::strategy_info_query::StrategyInfoQuery;
use crate::strategy_engine::strategy::Strategy;
use std::collections::HashMap;
use crate::strategy_engine::strategy::strategy_state_manager::StrategyRunState;
use crate::EngineName;
use async_trait::async_trait;
use crate::EngineContext;
use std::any::Any;






#[derive(Debug)]
pub struct StrategyEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub database: DatabaseConnection,
    pub strategy_list: HashMap<i32, Strategy>,
    pub market_event_receiver: broadcast::Receiver<Event>,
    pub request_event_receiver: broadcast::Receiver<Event>,
    pub response_event_receiver: broadcast::Receiver<Event>,

}


impl Clone for StrategyEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            strategy_list: self.strategy_list.clone(),
            database: self.database.clone(),
            market_event_receiver: self.market_event_receiver.resubscribe(),
            request_event_receiver: self.request_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),

        }
    }
}


#[async_trait]
impl EngineContext for StrategyEngineContext {

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
        let _event = event;
    }

}

impl StrategyEngineContext {
    pub async fn get_strategy_by_id(&self, id: i32) -> Result<StrategyInfo, String> {
        let strategy_info = StrategyInfoQuery::get_strategy_by_id(&self.database, id).await.unwrap();
        if let Some(strategy_info) = strategy_info {
            Ok(strategy_info)
        } else {
            tracing::error!("策略信息不存在");
            Err("策略信息不存在".to_string())
        }
    }

    pub async fn load_strategy_by_info(
        &mut self, 
        strategy_info: StrategyInfo
    ) -> Result<i32, String> {
        let strategy_id = strategy_info.id;
        let strategy = Strategy::new(
            strategy_info, 
            self.event_publisher.clone(), 
            self.market_event_receiver.resubscribe(), 
            self.response_event_receiver.resubscribe()
        ).await;
        self.strategy_list.insert(strategy_id, strategy);
        

        Ok(strategy_id)
    }


    pub async fn start_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.start_strategy().await.unwrap();
        Ok(())
    }



    // 设置策略
    pub async fn init_strategy(
        &mut self, 
        strategy_id: i32,
    ) -> Result<(), String> {
        let strategy_info = self.get_strategy_by_id(strategy_id).await?;
        // 加载策略（实例化策略）
        self.load_strategy_by_info(
            strategy_info
        ).await?;
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        // 获取策略的状态
        let strategy_state = strategy.state_manager.current_state();
        if strategy_state != StrategyRunState::Created {
            tracing::warn!("策略状态不是Created, 不设置策略");
            return Ok(());
        }
        strategy.init_strategy().await.unwrap();
        Ok(())
    }

    pub async fn stop_strategy(&mut self, strategy_id: i32) -> Result<(), String> {


        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.stop_strategy().await?;
        self.remove_strategy(strategy_id).await;


        Ok(())
    }

    async fn remove_strategy(&mut self, strategy_id: i32) {
        self.strategy_list.remove(&strategy_id);
        tracing::info!("策略实例已停止, 从引擎中移除, 策略名称: {}", strategy_id);
    }

    // 开启策略的事件推送
    pub async fn enable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.enable_strategy_event_push().await;
        Ok(())
    }

    pub async fn disable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.disable_event_push().await;
        Ok(())
    }

}
