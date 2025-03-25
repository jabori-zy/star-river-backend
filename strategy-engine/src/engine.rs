use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use tokio::sync::mpsc;
use sea_orm::DatabaseConnection;
use database::entities::strategy_info::Model as StrategyInfo;
use database::query::strategy_info_query::StrategyInfoQuery;
use crate::strategy::Strategy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::strategy::strategy_state_manager::StrategyRunState;


// 策略引擎
// 管理所有策略的执行
#[derive(Debug)]
pub struct StrategyEngine {
    market_event_receiver: broadcast::Receiver<Event>,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,
    strategy_event_receiver: broadcast::Receiver<Event>,
    event_publisher: EventPublisher,
    database: DatabaseConnection,
    strategy_list: Arc<Mutex<HashMap<i32, Strategy>>>,
    all_strategy_state: Arc<Mutex<HashMap<i32, StrategyRunState>>>,
    
}

impl Clone for StrategyEngine {
    fn clone(&self) -> Self {
        Self {
            market_event_receiver: self.market_event_receiver.resubscribe(),
            command_event_receiver: self.command_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            strategy_event_receiver: self.strategy_event_receiver.resubscribe(),
            event_publisher: self.event_publisher.clone(),
            database: self.database.clone(),
            strategy_list: self.strategy_list.clone(),
            all_strategy_state: self.all_strategy_state.clone(),
        }
    }
}

impl StrategyEngine {
    pub fn new(
        market_event_receiver: broadcast::Receiver<Event>,
        command_event_receiver: broadcast::Receiver<Event>, 
        response_event_receiver: broadcast::Receiver<Event>,
        strategy_event_receiver: broadcast::Receiver<Event>,
        event_publisher: EventPublisher,
        database: DatabaseConnection
    ) -> Self {
        Self {
            market_event_receiver,
            command_event_receiver,
            response_event_receiver,
            strategy_event_receiver,
            event_publisher,
            database,
            strategy_list: Arc::new(Mutex::new(HashMap::new())),
            all_strategy_state: Arc::new(Mutex::new(HashMap::new())),
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
        tracing::info!("策略引擎已启动");
        // let (internal_tx, internal_rx) = mpsc::channel(100);
        // self.listen(internal_tx).await;
        // self.handle_events(&self.event_publisher, internal_rx).await;
        // self.check_all_strategy_state().await?;
        Ok(())
    }

    pub async fn check_all_strategy_state(&self) -> Result<(), String> {
        let strategy_list = self.strategy_list.clone();
        let all_strategy_state = self.all_strategy_state.clone();
        tokio::spawn(async move {
            loop {
                let strategy_list = strategy_list.lock().await;
                let mut all_strategy_state = all_strategy_state.lock().await;
                if strategy_list.is_empty() {
                tracing::info!("没有策略运行");
            }

            for (strategy_id, strategy) in strategy_list.iter() {
                    let strategy_state = strategy.state_manager.current_state();
                    all_strategy_state.insert(strategy_id.clone(), strategy_state);
                }
                tracing::info!("所有策略状态: {:?}", all_strategy_state);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        Ok(())
    }

    // 自动初始化正在运行的策略
    // pub async fn init_strategy(&mut self) -> Result<(), String> {
    //     // 获取所有的可用策略
    //     let strategies = StrategyInfoQuery::get_all_strategy(&self.database).await.unwrap();
    //     for strategy in strategies {
    //         tracing::info!("添加策略: {}", strategy.name);
    //         self.create_strategy_by_info(strategy).await.unwrap();
    //     }
    //     Ok(())
    // }

    pub async fn get_strategy_by_id(&self, id: i32) -> Result<StrategyInfo, String> {
        let strategy_info = StrategyInfoQuery::get_strategy_by_id(&self.database, id).await.unwrap();
        if let Some(strategy_info) = strategy_info {
            Ok(strategy_info)
        } else {
            tracing::error!("策略信息不存在");
            Err("策略信息不存在".to_string())
        }
        
    }

    pub async fn load_strategy_by_info(&mut self, strategy_info: StrategyInfo) -> Result<i32, String> {
        let strategy_id = strategy_info.id;
        let strategy = Strategy::new(
            strategy_info, 
            self.event_publisher.clone(), 
            self.market_event_receiver.resubscribe(), 
            self.response_event_receiver.resubscribe(),
            self.strategy_event_receiver.resubscribe(),
        ).await;
        let mut strategy_list = self.strategy_list.lock().await;
        strategy_list.insert(strategy_id, strategy);
        

        Ok(strategy_id)
    }

    // 设置策略
    pub async fn init_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy_info = self.get_strategy_by_id(strategy_id).await?;
        // 加载策略（实例化策略）
        self.load_strategy_by_info(strategy_info).await?;
        let mut strategy_list = self.strategy_list.lock().await;
        let strategy = strategy_list.get_mut(&strategy_id).unwrap();
        // 获取策略的状态
        let strategy_state = strategy.state_manager.current_state();
        if strategy_state != StrategyRunState::Created {
            tracing::warn!("策略状态不是Created, 不设置策略");
            return Ok(());
        }
        strategy.init_strategy().await.unwrap();
        Ok(())
    }

    // 启动策略
    pub async fn start_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut strategy_list = self.strategy_list.lock().await;
        let strategy = strategy_list.get_mut(&strategy_id).unwrap();
        strategy.start_strategy().await.unwrap();
        Ok(())
    }

    // 停止策略
    pub async fn stop_strategy(&mut self, strategy_id: i32) -> Result<(), String> {

        let strategy_name = {
            let mut strategy_list = self.strategy_list.lock().await;
            let strategy = strategy_list.get_mut(&strategy_id).unwrap();
            strategy.stop_strategy().await?;
            strategy.strategy_name.clone()
        };
        

        self.remove_strategy(strategy_id, strategy_name).await;


        Ok(())
    }

    async fn remove_strategy(&mut self, strategy_id: i32, strategy_name: String) {
        let mut strategy_list = self.strategy_list.lock().await;
        strategy_list.remove(&strategy_id);
        tracing::info!("策略实例已停止, 从引擎中移除, 策略名称: {}", strategy_name);
    }

    // 开启策略的事件推送
    pub async fn enable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut strategy_list = self.strategy_list.lock().await;
        let strategy = strategy_list.get_mut(&strategy_id).unwrap();
        strategy.enable_strategy_event_push().await;
        Ok(())
    }

    pub async fn disable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut strategy_list = self.strategy_list.lock().await;
        let strategy = strategy_list.get_mut(&strategy_id).unwrap();
        strategy.disable_event_push();
        Ok(())
    }
    
}