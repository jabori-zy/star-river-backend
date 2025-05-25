use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use serde::Deserialize;
use axum::extract::{Json,Query,Path};
use crate::api::response::ApiResponse;
use types::engine::EngineName;
use engine::strategy_engine::StrategyEngine;

#[derive(Deserialize, Debug)]
pub struct SetupStrategyParams {
    pub strategy_id: i32,
}

// 初始化策略
pub async fn init_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
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
pub async fn run_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
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

pub async fn stop_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let heartbeat = star_river.heartbeat.lock().await;
    heartbeat.run_async_task_once("停止策略".to_string(), async move {
        let engine_manager = star_river.engine_manager.lock().await;
        let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
        let mut engine_guard = engine.lock().await;
        let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
        strategy_engine.stop_strategy(strategy_id).await.unwrap();
    }).await;
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}

// #[derive(Deserialize, Debug)]
// pub struct DisableStrategyEventPushParams {
//     pub strategy_id: i32,
// }

// pub async fn disable_strategy_event_push(State(star_river): State<StarRiver>, Json(params): Json<DisableStrategyEventPushParams>) -> (StatusCode, Json<ApiResponse<()>>) {
//     let strategy_id = params.strategy_id;
//     let engine_manager = star_river.engine_manager.lock().await;
//     let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
//     let mut engine_guard = engine.lock().await;
//     let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
//     strategy_engine.disable_strategy_event_push(strategy_id).await.expect("关闭策略事件推送失败");
//     (StatusCode::OK, Json(ApiResponse {
//         code: 0,
//         message: "success".to_string(),
//         data: None,
//     }))
// }


#[derive(Deserialize, Debug)]
pub struct GetStrategyCacheKeysParams {
    pub strategy_id: i32,
}


pub async fn get_strategy_cache_keys(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<Vec<String>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let cache_keys = strategy_engine.get_strategy_cache_keys(strategy_id).await;
    let cache_keys_str = cache_keys.iter().map(|cache_key| cache_key.get_key()).collect();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: Some(cache_keys_str),
    }))
}


#[derive(Deserialize, Debug)]
pub struct EnableStrategyDataPushParams {
    pub strategy_id: i32,
}

pub async fn enable_strategy_data_push(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.enable_strategy_data_push(strategy_id).await.expect("开启策略数据推送失败");


    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[derive(Deserialize, Debug)]
pub struct DisableStrategyDataPushParams {
    pub strategy_id: i32,
}

pub async fn disable_strategy_data_push(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.disable_strategy_data_push(strategy_id).await.expect("关闭策略数据推送失败");


    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[derive(Deserialize, Debug)]
pub struct PlayParams {
    pub strategy_id: i32,
}
pub async fn play(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.play(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[derive(Deserialize, Debug)]
pub struct PauseParams {
    pub strategy_id: i32,
}

pub async fn pause(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.pause(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[derive(Deserialize, Debug)]
pub struct StopParams {
    pub strategy_id: i32,
}

pub async fn stop(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.stop(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[derive(Deserialize, Debug)]
pub struct PlayOneParams {
    pub strategy_id: i32,
}

pub async fn play_one(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.play_one_kline(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}