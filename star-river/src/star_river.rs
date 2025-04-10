use event_center::{EventCenter, Channel};
use heartbeat::Heartbeat;
use database::DatabaseManager;
use tokio::sync::Mutex;
use std::sync::Arc;
use axum::extract::State;
use engine::engine_manager::EngineManager;



#[derive(Clone)]
pub struct StarRiver {
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub event_center: Arc<Mutex<EventCenter>>,
    pub database: Arc<Mutex<DatabaseManager>>,
    pub engine_manager: Arc<Mutex<EngineManager>>,
}

impl StarRiver {
    pub async fn new() -> Self {
        
        let event_center = EventCenter::new();
        // 初始化数据库
        let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
        let database_event_publisher = event_center.get_event_publisher();
        let database = DatabaseManager::new(command_event_receiver, database_event_publisher).await;

        // 初始化引擎管理器
        let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).unwrap();
        let market_event_receiver = event_center.subscribe(&Channel::Market).unwrap();
        let request_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
        let engine_manager = EngineManager::new(
            event_center.get_event_publisher(), 
            exchange_event_receiver, 
            market_event_receiver, 
            request_event_receiver, 
            response_event_receiver, 
            database.get_conn());

        Self {
            heartbeat: Arc::new(Mutex::new(Heartbeat::new(1000))),
            event_center: Arc::new(Mutex::new(event_center)),
            database: Arc::new(Mutex::new(database)),
            engine_manager: Arc::new(Mutex::new(engine_manager)),
        }
    }
}


pub async fn init_app(State(app_state): State<StarRiver>) {
    // 初始化app
    start_heartbeat(State(app_state.clone())).await;
    start_database(State(app_state.clone())).await;
    start_engine_manager(State(app_state.clone())).await;
}


async fn start_heartbeat(star_river: State<StarRiver>) {
    let heartbeat = star_river.heartbeat.clone();
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
        tracing::info!("心跳已启动");
    });
}



async fn start_database(star_river: State<StarRiver>) {
    let database = star_river.database.clone();
    tokio::spawn(async move {
        let database = database.lock().await;
        database.start().await;
    });
}


async fn start_engine_manager(star_river: State<StarRiver>) {
    let engine_manager = star_river.engine_manager.clone();
    tokio::spawn(async move {
        let engine_manager = engine_manager.lock().await;
        engine_manager.start().await;
    });
}
