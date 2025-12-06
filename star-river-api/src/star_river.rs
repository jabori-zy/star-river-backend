use std::sync::Arc;

use axum::extract::State;
use database::{DatabaseManager, query::system_config_query::SystemConfigQuery};
// use event_center::EventCenterSingleton;
use event_center::EventCenterSingleton;
use heartbeat::Heartbeat;
use star_river_core::system::system_config::SystemConfigManager;
use tokio::sync::Mutex;
use tracing::instrument;

use super::EngineManager;

#[derive(Clone, Debug)]
pub struct StarRiver {
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    // pub event_center: Arc<Mutex<EventCenter>>,
    pub database: Arc<Mutex<DatabaseManager>>,
    pub engine_manager: Arc<Mutex<EngineManager>>,
}

impl StarRiver {
    pub async fn new() -> Self {
        // System heartbeat interval is 100 milliseconds
        let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

        // let mut event_center = EventCenter::new().init_channel().await;
        // EventCenterSingleton::init().await.unwrap();
        EventCenterSingleton::init().unwrap();
        // Initialize database

        let database = DatabaseManager::new().await;

        // Initialize engine manager
        let engine_manager = EngineManager::new(
            // &mut event_center,
            database.get_conn(),
            heartbeat.clone(),
        )
        .await;

        let system_config = SystemConfigQuery::get_system_config(&database.get_conn()).await.unwrap();
        // Initialize timezone
        SystemConfigManager::initialize_from_db(system_config);
        let timezone = SystemConfigManager::get_timezone();
        tracing::info!("current timezone: {}", timezone);

        Self {
            heartbeat: heartbeat.clone(),
            // event_center: Arc::new(Mutex::new(event_center)),
            database: Arc::new(Mutex::new(database)),
            engine_manager: Arc::new(Mutex::new(engine_manager)),
        }
    }
}

#[instrument]
pub async fn init_app(State(app_state): State<StarRiver>) {
    // Initialize application
    start_heartbeat(State(app_state.clone())).await;
    start_engine_manager(State(app_state.clone())).await;
}

async fn start_heartbeat(star_river: State<StarRiver>) {
    let heartbeat = star_river.heartbeat.clone();
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
        tracing::info!("Heartbeat started");
    });
}

async fn start_engine_manager(star_river: State<StarRiver>) {
    let engine_manager = star_river.engine_manager.clone();
    tokio::spawn(async move {
        let engine_manager = engine_manager.lock().await;
        engine_manager.start().await;
    });
}
