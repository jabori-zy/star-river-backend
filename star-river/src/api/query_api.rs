use axum::extract::{State, Query, Json};

use database::query::strategy_config_query::StrategyConfigQuery;
use database::entities::strategy_config;
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use database::query::account_config_query::AccountConfigQuery;
use types::account::AccountConfig;
use crate::api::response::ApiResponse;

#[derive(Serialize, Deserialize)]
pub struct GetStrategyListParams {
    pub page: Option<u64>,
    pub strategy_per_page: Option<u64>,
}

#[derive(Serialize)]
pub struct GetStrategyListResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<Vec<strategy_config::Model>>,
    pub page_num: Option<u64>,
}

#[axum::debug_handler]
pub async fn get_strategy_list(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetStrategyListParams>,
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


#[derive(Serialize, Deserialize)]
pub struct GetStrategyByIdParams {
    pub id: i32,
}

#[derive(Serialize)]
pub struct GetStrategyByIdResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<strategy_config::Model>,
}

pub async fn get_strategy_by_id(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetStrategyByIdParams>,
) -> (StatusCode, Json<GetStrategyByIdResponse>) {
    let db = &star_river.database.lock().await.conn;
    let strategy = StrategyConfigQuery::get_strategy_by_id(db, params.id).await.unwrap();
    (
        StatusCode::OK,
        Json(
            GetStrategyByIdResponse {
                code: 0,
                message: "获取成功".to_string(),
                data: strategy,
            }
        )
    )
}


pub async fn get_account_config(
    State(star_river): State<StarRiver>
) -> (StatusCode, Json<ApiResponse<Vec<AccountConfig>>>) {
    let db = &star_river.database.lock().await.conn;
    let account_config = AccountConfigQuery::get_all_account_config(db).await.unwrap();
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(account_config),
        })
    )
}


#[derive(Serialize, Deserialize)]
pub struct GetAccountConfigByExchangeParams {
    pub exchange: String,
}

pub async fn get_account_config_by_exchange(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetAccountConfigByExchangeParams>,
) -> (StatusCode, Json<ApiResponse<Vec<AccountConfig>>>) {
    let db = &star_river.database.lock().await.conn;
    let account_config = AccountConfigQuery::get_account_config_by_exchange(db, params.exchange).await.unwrap();
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(account_config),
        })
    )
}
