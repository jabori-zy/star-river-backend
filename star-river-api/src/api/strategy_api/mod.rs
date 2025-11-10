pub mod backtest;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use backtest_engine::engine_error::{BacktestEngineError, StrategyConfigNotFoundSnafu};
use database::{mutation::strategy_config_mutation::StrategyConfigMutation, query::strategy_config_query::StrategyConfigQuery};
use engine_core::EngineContextAccessor;
use serde::{Deserialize, Serialize};
use snafu::IntoError;
use strategy_core::strategy::StrategyConfig;
use tracing::instrument;
use utoipa::{IntoParams, ToSchema};

use crate::{
    api::response::{ApiResponse, NewApiResponse},
    star_river::StarRiver,
};

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
    pub data: Option<Vec<StrategyConfig>>,
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
    let (strategy_list, num_pages) = StrategyConfigQuery::get_strategy_list_in_page(db, page, strategy_per_page)
        .await
        .unwrap();
    (
        StatusCode::OK,
        Json(GetStrategyListResponse {
            code: 0,
            message: "获取成功".to_string(),
            page_num: Some(num_pages),
            data: Some(strategy_list),
        }),
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
        (status = 200, body = NewApiResponse<StrategyConfig>),
        (status = 400, body = NewApiResponse<StrategyConfig>)
    )
)]
pub async fn get_strategy_by_id(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<StrategyConfig>>) {
    let db = &star_river.database.lock().await.conn;
    let strategy = StrategyConfigQuery::get_strategy_by_id(db, strategy_id).await;
    if let Ok(strategy) = strategy {
        (StatusCode::OK, Json(NewApiResponse::success(strategy)))
    } else {
        let error = StrategyConfigNotFoundSnafu { strategy_id }.into_error(strategy.unwrap_err());
        (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(error)))
    }
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
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/strategy",
    tag = "策略管理",
    summary = "创建策略",
    responses(
        (status = 200, body = ApiResponse<StrategyConfig>),
        (status = 400, body = ApiResponse<String>)
    )
)]
pub async fn create_strategy(
    State(star_river): State<StarRiver>,
    Json(params): Json<CreateStrategyParams>,
) -> (StatusCode, Json<ApiResponse<StrategyConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::create_strategy(conn, params.name, params.description).await {
        Ok(strategy) => {
            tracing::info!("创建策略成功: {:?}", strategy);
            // 创建策略系统变量
            // if let Err(e) = StrategySysVariableMutation::insert_strategy_sys_variable(conn, strategy.id).await {
            //     tracing::error!("创建策略系统变量失败: {:?}", e);
            // }
            (
                StatusCode::CREATED,
                Json(ApiResponse {
                    code: 0,
                    message: "创建成功".to_string(),
                    data: Some(strategy),
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}

#[derive(Serialize, Deserialize, Debug, IntoParams, ToSchema)]
#[schema(title = "更新策略参数", description = "更新策略参数")]
pub struct UpdateStrategyParams {
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略交易模式
    pub trade_mode: String,
    /// 策略配置
    pub config: Option<serde_json::Value>,
    /// 策略节点
    pub nodes: Option<serde_json::Value>,
    /// 策略边
    pub edges: Option<serde_json::Value>,
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
        (status = 200, body = ApiResponse<StrategyConfig>),
        (status = 400, body = ApiResponse<String>)
    )
)]
pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Json(params): Json<UpdateStrategyParams>,
) -> (StatusCode, Json<ApiResponse<StrategyConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_strategy_by_id(
        conn,
        strategy_id,
        params.name,
        params.description,
        params.trade_mode,
        params.config,
        params.nodes,
        params.edges,
    )
    .await
    {
        Ok(strategy) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(strategy),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
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
pub async fn delete_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::delete_strategy(conn, strategy_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "策略删除成功".to_string(),
                data: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}

// 初始化策略
#[utoipa::path(
    post,
    path = "/api/v1/strategy/{strategy_id}/init",
    tag = "Strategy Management",
    summary = "Initialize strategy",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to initialize")
    ),
    responses(
        (status = OK, description = "Initialize strategy successfully", content_type = "application/json"),
    )
)]
#[instrument(skip(star_river))]
pub async fn init_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    tracing::info!(strategy_id = strategy_id, "initialize strategy");
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard
        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.init(strategy_id).await }))
        .await;

    if let Err(e) = result {
        return (StatusCode::CONFLICT, Json(NewApiResponse::error(e)));
    }

    (StatusCode::OK, Json(NewApiResponse::success(())))
}

#[utoipa::path(
    post,
    path = "/api/v1/strategy/{strategy_id}/stop",
    tag = "Strategy Management",
    summary = "stop strategy",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to stop")
    ),
    responses(
        (status = 200, description = "Stop strategy successfully", content_type = "application/json")
    )
)]
#[instrument(skip(star_river))]
pub async fn stop_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    tracing::info!(strategy_id = strategy_id, "stop strategy");
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard
        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.stop(strategy_id).await }))
        .await;

    if let Err(e) = result {
        let status_code = match &e {
            BacktestEngineError::StrategyInstanceNotFound { .. } => StatusCode::NOT_FOUND,
            BacktestEngineError::StrategyConfigNotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        return (status_code, Json(NewApiResponse::error(e)));
    }

    (StatusCode::OK, Json(NewApiResponse::success(())))
}
