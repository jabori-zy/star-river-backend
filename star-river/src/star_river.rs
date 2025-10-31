use super::EngineManager;
use axum::extract::State;
use database::DatabaseManager;
use database::query::system_config_query::SystemConfigQuery;
use event_center::EventCenterSingleton;
use heartbeat::Heartbeat;
use star_river_core::system::system_config::SystemConfigManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct StarRiver {
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    // pub event_center: Arc<Mutex<EventCenter>>,
    pub database: Arc<Mutex<DatabaseManager>>,
    pub engine_manager: Arc<Mutex<EngineManager>>,
}

impl StarRiver {
    pub async fn new() -> Self {
        // 系统心跳间隔为100毫秒
        let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

        // let mut event_center = EventCenter::new().init_channel().await;
        EventCenterSingleton::init().await.unwrap();
        // 初始化数据库

        let database = DatabaseManager::new().await;

        // 初始化引擎管理器
        let engine_manager = EngineManager::new(
            // &mut event_center,
            database.get_conn(),
            heartbeat.clone(),
        )
        .await;

        let system_config = SystemConfigQuery::get_system_config(&database.get_conn()).await.unwrap();
        tracing::info!("system_config: {:?}", system_config);
        // 初始化时区
        SystemConfigManager::initialize_from_db(system_config);
        // 添加这行确认
        let config_after_init = SystemConfigManager::get_config();
        tracing::info!("config after init: {:?}", config_after_init);

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
