use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use serde::Deserialize;
use axum::extract::Json;
use crate::api::response::ApiResponse;
use engine::EngineName;
use engine::strategy_engine::StrategyEngine;


#[derive(Deserialize, Debug)]
pub struct SetupStrategyParams {
    pub strategy_id: i32,
}

// 初始化策略
pub async fn init_strategy(State(star_river): State<StarRiver>, Json(params): Json<SetupStrategyParams>) -> (StatusCode, Json<ApiResponse<()>>) {
    let strategy_id = params.strategy_id;
    let heartbeat = star_river.heartbeat.lock().await;
    heartbeat.run_async_task_once(format!("设置策略{}", strategy_id), async move {
        let engine_manager = star_river.engine_manager.lock().await;
        let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
        let mut engine_guard = engine.lock().await;
        let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
        strategy_engine.init_strategy(strategy_id).await.unwrap();
    }).await;
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}






#[derive(Deserialize, Debug)]
pub struct RunStrategyParams {
    pub strategy_id: i32,
}

// todo
// 将strategy_engine 中将策略的启动逻辑拆分为两部分：
// start_strategy: 负责策略的初始化和启动，确保策略可以正常运行（比如检查配置、建立连接等）
// 实际的策略运行逻辑应该在一个单独的异步任务中进行
// 例如，strategy_engine 的实现可能是这样的
pub async fn run_strategy(State(star_river): State<StarRiver>, Json(params): Json<RunStrategyParams>) -> (StatusCode, Json<ApiResponse<()>>) {
    let strategy_id = params.strategy_id;
    let heartbeat = star_river.heartbeat.lock().await;
    heartbeat.run_async_task_once(format!("启动策略{}", strategy_id), async move {
        let engine_manager = star_river.engine_manager.lock().await;
        let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
        let mut engine_guard = engine.lock().await;
        let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
        strategy_engine.start_strategy(strategy_id).await.unwrap();
    }).await;
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}

#[derive(Deserialize, Debug)]
pub struct StopStrategyParams {
    pub strategy_id: i32,
}

pub async fn stop_strategy(State(star_river): State<StarRiver>, Json(params): Json<StopStrategyParams>) -> (StatusCode, Json<ApiResponse<()>>) {
    let  heartbeat = star_river.heartbeat.lock().await;
    heartbeat.run_async_task_once("停止策略".to_string(), async move {
        let engine_manager = star_river.engine_manager.lock().await;
        let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
        let mut engine_guard = engine.lock().await;
        let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
        strategy_engine.stop_strategy(params.strategy_id).await.unwrap();
    }).await;
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}

// 开启策略的事件推送
#[derive(Deserialize, Debug)]
pub struct EnableStrategyEventPushParams {
    pub strategy_id: i32,
}

pub async fn enable_strategy_event_push(State(star_river): State<StarRiver>, Json(params): Json<EnableStrategyEventPushParams>) -> (StatusCode, Json<ApiResponse<()>>) {
    let strategy_id = params.strategy_id;
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.enable_strategy_event_push(strategy_id).await.expect("开启策略事件推送失败");
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}

#[derive(Deserialize, Debug)]
pub struct DisableStrategyEventPushParams {
    pub strategy_id: i32,
}

pub async fn disable_strategy_event_push(State(star_river): State<StarRiver>, Json(params): Json<DisableStrategyEventPushParams>) -> (StatusCode, Json<ApiResponse<()>>) {
    let strategy_id = params.strategy_id;
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.disable_strategy_event_push(strategy_id).await.expect("关闭策略事件推送失败");
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


