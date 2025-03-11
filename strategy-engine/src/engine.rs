use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use tokio::sync::mpsc;
use event_center::command_event::{CommandEvent, DatabaseCommand, CreateStrategyParams};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use database::entities::strategy_info::Model as StrategyInfo;
use database::query::strategy_info_query::StrategyInfoQuery;
use crate::strategy::Strategy;
use std::collections::HashMap;


// 策略引擎
// 管理所有策略的执行
#[derive(Debug)]
pub struct StrategyEngine {
    market_event_receiver: broadcast::Receiver<Event>,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,
    event_publisher: EventPublisher,
    database: DatabaseConnection,
    strategy_list: HashMap<i32, Strategy>
    
}

impl Clone for StrategyEngine {
    fn clone(&self) -> Self {
        Self {
            market_event_receiver: self.market_event_receiver.resubscribe(),
            command_event_receiver: self.command_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            event_publisher: self.event_publisher.clone(),
            database: self.database.clone(),
            strategy_list: self.strategy_list.clone(),
        }
    }
}

impl StrategyEngine {
    pub fn new(
        market_event_receiver: broadcast::Receiver<Event>,
        command_event_receiver: broadcast::Receiver<Event>, 
        response_event_receiver: broadcast::Receiver<Event>,
        event_publisher: EventPublisher,
        database: DatabaseConnection
    ) -> Self {
        Self {
            market_event_receiver,
            command_event_receiver,
            response_event_receiver,
            event_publisher,
            database,
            strategy_list: HashMap::new(),
        }
    }

    async fn listen(&self, internal_tx: mpsc::Sender<Event>) {
        tracing::info!("策略引擎启动成功, 开始监听...");
        let mut command_receiver = self.command_event_receiver.resubscribe();
        let mut response_receiver = self.response_event_receiver.resubscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Ok(event) = command_receiver.recv() => {
                    //     let _ = internal_tx.send(event).await;
                    // }
                    Ok(event) = response_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });

    }
    
    async fn handle_events(event_publisher: &EventPublisher, mut internal_rx: mpsc::Receiver<Event>) {
        loop {
            // let event = internal_rx.recv().await.unwrap();
            // match event {
            //     Event::Response(response_event) => {
            //         self.handle_response_event(response_event).await;
            //     }
            // }
        }
    }

    pub async fn start(& self) -> Result<(), String> {
        let (internal_tx, internal_rx) = mpsc::channel(100);
        self.listen(internal_tx).await;
        // self.handle_events(&self.event_publisher, internal_rx).await;
        Ok(())
    }

    //创建策略
    pub async fn create_strategy(&self, name: String, description: String) -> Result<(), String> {
        let params = CreateStrategyParams {
            name,
            description,
        };
        let database_command = DatabaseCommand::CreateStrategy(params);
        let command = Event::Command(CommandEvent::Database(database_command));
        // tracing::info!("创建策略: {:?}", command);
        let _ = self.event_publisher.publish(command);
        Ok(())
    }

    pub async fn get_strategy_by_id(&self, id: i32) -> Result<StrategyInfo, String> {
        let strategy_info = StrategyInfoQuery::get_strategy_by_id(&self.database, id).await.unwrap();
        if let Some(strategy_info) = strategy_info {
            Ok(strategy_info)
        } else {
            tracing::error!("策略信息不存在");
            Err("策略信息不存在".to_string())
        }
        
    }

    pub async fn create_strategy_by_info(&mut self, strategy_info: StrategyInfo) -> Result<i32, String> {
        let strategy_id = strategy_info.id;

        let strategy = Strategy::new(strategy_info, self.event_publisher.clone(), self.market_event_receiver.resubscribe(), self.response_event_receiver.resubscribe()).await;
        self.strategy_list.insert(strategy_id, strategy);

        Ok(strategy_id)
    }

    pub async fn run_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.run().await;
        Ok(())
    }
    
    
}