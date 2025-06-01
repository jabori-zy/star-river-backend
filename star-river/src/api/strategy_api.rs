use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use serde::{Serialize, Deserialize};
use axum::extract::{Json,Query,Path};
use crate::api::response::ApiResponse;
use types::engine::EngineName;
use engine::strategy_engine::StrategyEngine;
use utoipa::{IntoParams, ToSchema};
use types::strategy::Strategy;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::mutation::strategy_sys_variable_mutation::StrategySysVariableMutation;
use database::query::strategy_config_query::StrategyConfigQuery;



#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "获取策略列表参数",
    description = "获取策略列表参数",
    example = json!({
        "page": 1,
        "strategy_per_page": 10
    })
)]
pub struct GetStrategyListQuery {
    /// 页码
    #[schema(example = 1, minimum = 1)]
    pub page: Option<u64>,
    /// 每页策略数量
    #[schema(example = 10, minimum = 10)]
    pub strategy_per_page: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct GetStrategyListResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<Vec<Strategy>>,
    pub page_num: Option<u64>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/v1/strategy",
    tag = "策略管理",
    summary = "获取策略列表",
    params(GetStrategyListQuery),
    responses(
        (status = 200, body = GetStrategyListResponse),
        (status = 400, body = GetStrategyListResponse)
    )
)]
pub async fn get_strategy_list(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetStrategyListQuery>,
) -> (StatusCode, Json<GetStrategyListResponse>) {
    let page = params.page.unwrap_or(1);
    let strategy_per_page = params.strategy_per_page.unwrap_or(10);

    let db = &star_river.database.lock().await.conn;
    let (strategy_list, num_pages) = StrategyConfigQuery::get_strategy_list_in_page(db, page, strategy_per_page).await.unwrap();
    (
        StatusCode::OK,
        Json(GetStrategyListResponse {
            code: 0,
            message: "获取成功".to_string(),
            page_num: Some(num_pages),
            data: Some(strategy_list),
        })
    )
}



#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/v1/strategy/{strategy_id}",
    tag = "策略管理",
    summary = "获取策略详情",
    params(
        ("strategy_id" = i32, Path, description = "策略ID")
    ),
    responses(
        (status = 200, body = ApiResponse<Strategy>),
        (status = 400, body = ApiResponse<Strategy>)
    )
)]
pub async fn get_strategy_by_id(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<Strategy>>) {
    let db = &star_river.database.lock().await.conn;
    let strategy = StrategyConfigQuery::get_strategy_by_id(db, strategy_id).await.unwrap();
    (
        StatusCode::OK,
        Json(
            ApiResponse {
                code: 0,
                message: "获取成功".to_string(),
                data: strategy,
            }
        )
    )
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "创建策略参数",
    description = "创建策略参数",
    example = json!({
        "name": "测试策略1",
        "description": "测试策略1描述",
        "status": 1
    })
)]
pub struct CreateStrategyParams {
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略状态
    pub status: i32,
}


#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/strategy",
    tag = "策略管理",
    summary = "创建策略",
    responses(
        (status = 200, body = ApiResponse<Strategy>),
        (status = 400, body = ApiResponse<String>)
    )
)]
pub async fn create_strategy(
    State(star_river): State<StarRiver>,
    Json(params): Json<CreateStrategyParams>,
) -> (StatusCode, Json<ApiResponse<Strategy>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::create_strategy(conn, params.name, params.description, params.status).await {
        Ok(strategy) => {
            tracing::info!("创建策略成功: {:?}", strategy);
            // 创建策略系统变量
            if let Err(e) = StrategySysVariableMutation::insert_strategy_sys_variable(conn, strategy.id).await {
                tracing::error!("创建策略系统变量失败: {:?}", e);
            }
            (
            StatusCode::CREATED,
            Json(ApiResponse {
                code: 0,
                message: "创建成功".to_string(),
                data: Some(strategy),
            })
        )
    },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}

#[derive(Serialize, Deserialize,Debug, IntoParams, ToSchema)]
#[schema(
    title = "更新策略参数",
    description = "更新策略参数"
)]
pub struct UpdateStrategyParams {
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略交易模式
    pub trade_mode: String,
    /// 策略状态
    pub status: i32,
    /// 策略配置
    pub config: Option<serde_json::Value>,
    /// 策略节点
    pub nodes: Option<serde_json::Value>,
    /// 策略边
    pub edges: Option<serde_json::Value>,
    /// 策略图表配置
    pub chart_config: Option<serde_json::Value>,
}

#[utoipa::path(
    put,
    path = "/api/v1/strategy/{strategy_id}",
    tag = "策略管理",
    summary = "更新策略",
    params(
        ("strategy_id" = i32, Path, description = "要更新的策略ID"),
        UpdateStrategyParams
    ),
    responses(
        (status = 200, body = ApiResponse<Strategy>),
        (status = 400, body = ApiResponse<String>)
    )
)]
pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Json(params): Json<UpdateStrategyParams>,
) -> (StatusCode, Json<ApiResponse<Strategy>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_strategy_by_id(
        conn,
        strategy_id,
        params.name, 
        params.description, 
        params.trade_mode,
        params.status, 
        params.config,
        params.nodes, 
        params.edges,
        params.chart_config,
    ).await {
        Ok(strategy) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(strategy),
            })
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}


#[axum::debug_handler]
#[utoipa::path(
    delete,
    path = "/api/v1/strategy/{strategy_id}",
    tag = "策略管理",
    summary = "删除策略",
    params(
        ("strategy_id" = i32, Path, description = "要删除的策略ID")
    ),
    responses(
        (status = 200, description = "策略删除成功", content_type = "application/json" ),
        (status = 400, description = "策略删除失败", content_type = "application/json")
    )
)]
pub async fn delete_strategy(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::delete_strategy(conn, strategy_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "策略删除成功".to_string(),
                data: None,
            })
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}



// 初始化策略
#[utoipa::path(
    post,
    path = "/api/v1/strategy/{strategy_id}/init",
    tag = "策略控制",
    summary = "初始化策略",
    params(
        ("strategy_id" = i32, Path, description = "初始化的策略ID")
    ),
    responses(
        (status = 200, description = "初始化策略成功", content_type = "application/json")
    )
)]
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


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/play",
    tag = "策略控制",
    summary = "播放k线",
    params(
        ("strategy_id" = i32, Path, description = "要播放的策略ID")
    ),
    responses(
        (status = 200, description = "播放策略成功")
    )
)]
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


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/pause",
    tag = "策略控制",
    summary = "暂停播放k线",
    params(
        ("strategy_id" = i32, Path, description = "要暂停的策略ID")
    ),
    responses(
        (status = 200, description = "暂停策略成功"),
        (status = 400, description = "暂停策略失败")
    )
)]
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


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/stop",
    tag = "策略控制",
    summary = "停止播放k线",
    params(
        ("strategy_id" = i32, Path, description = "要停止的策略ID")
    ),
    responses(
        (status = 200, description = "停止策略成功"),
        (status = 400, description = "停止策略失败")
    )
)]
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



#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/play_one",
    tag = "策略控制",
    summary = "播放单个K线",
    params(
        ("strategy_id" = i32, Path, description = "要播放单个K线的策略ID")
    ),
    responses(
        (status = 200, description = "播放单个K线成功"),
        (status = 400, description = "播放单个K线失败")
    )
)]
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