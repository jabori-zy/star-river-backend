use axum::extract::{Json, Query, State};

use database::mutation::strategy_info_mutation::StrategyInfoMutation;
use database::entities::strategy_info;
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;

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
) -> (StatusCode, Json<ApiResponse<strategy_info::Model>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyInfoMutation::create_strategy(conn, request.name, request.description, request.status).await {
        Ok(strategy) => {
            tracing::info!("创建策略成功: {:?}", strategy);
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
    pub status: i32,
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
}

pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateStrategyRequest>,
) -> (StatusCode, Json<ApiResponse<strategy_info::Model>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyInfoMutation::update_strategy_by_id(conn, request.id, request.name, request.description, request.status, request.nodes, request.edges).await {
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
    match StrategyInfoMutation::delete_strategy(conn, request.id).await {
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

