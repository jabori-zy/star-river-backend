pub mod account_mutation;


use axum::extract::{Json, Query, State};

use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::entities::strategy_config;
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;
use database::mutation::strategy_sys_variable_mutation::StrategySysVariableMutation;
use types::strategy::Strategy;

#[derive(Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub description: String,
    pub status: i32,
}


#[axum::debug_handler]
pub async fn create_strategy(
    State(star_river): State<StarRiver>,
    Json(request): Json<CreateStrategyRequest>,
) -> (StatusCode, Json<ApiResponse<Strategy>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::create_strategy(conn, request.name, request.description, request.status).await {
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

#[derive(Serialize, Deserialize,Debug)]
pub struct UpdateStrategyRequest {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub trade_mode: String,
    pub status: i32,
    pub config: Option<serde_json::Value>,
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
    pub chart_config: Option<serde_json::Value>,
}

pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateStrategyRequest>,
) -> (StatusCode, Json<ApiResponse<Strategy>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_strategy_by_id(
        conn,
        request.id, 
        request.name, 
        request.description, 
        request.trade_mode,
        request.status, 
        request.config,
        request.nodes, 
        request.edges,
        request.chart_config,
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

#[derive(Serialize, Deserialize)]
pub struct DeleteStrategyRequest {
    pub id: i32,
}

pub async fn delete_strategy(
    State(star_river): State<StarRiver>,
    Query(request): Query<DeleteStrategyRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::delete_strategy(conn, request.id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "删除成功".to_string(),
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

