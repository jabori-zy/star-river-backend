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

        // 系统心跳间隔为100毫秒
        let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100))); 
        
        let event_center = EventCenter::new();
        // 初始化数据库

        let database = DatabaseManager::new().await;

        // 初始化引擎管理器
        let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).unwrap();
        let market_event_receiver = event_center.subscribe(&Channel::Market).unwrap();
        let request_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
        let account_event_receiver = event_center.subscribe(&Channel::Account).unwrap();
        let engine_manager = EngineManager::new(
            event_center.get_event_publisher(), 
            exchange_event_receiver, 
            market_event_receiver, 
            request_event_receiver, 
            response_event_receiver, 
            account_event_receiver,
            database.get_conn(),
            heartbeat.clone()
        );

        Self {
            heartbeat: heartbeat.clone(),
            event_center: Arc::new(Mutex::new(event_center)),
            database: Arc::new(Mutex::new(database)),
            engine_manager: Arc::new(Mutex::new(engine_manager)),
        }
    }
}


pub async fn init_app(State(app_state): State<StarRiver>) {
    // 初始化app
    start_heartbeat(State(app_state.clone())).await;
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


async fn start_engine_manager(star_river: State<StarRiver>) {
    let engine_manager = star_river.engine_manager.clone();
    tokio::spawn(async move {
        let engine_manager = engine_manager.lock().await;
        engine_manager.start().await;
    });
}
