use axum::extract::{Json, Query, State};

use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::entities::strategy_config;
use types::account::AccountConfig;
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;
use database::mutation::mt5_account_config_mutation::Mt5AccountConfigMutation;
use types::market::Exchange;
use std::str::FromStr;
use types::account::mt5_account::Mt5AccountConfig;
use event_center::Event;
use event_center::account_event::AccountEvent;
use types::account::ExchangeAccountConfig;

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
) -> (StatusCode, Json<ApiResponse<strategy_config::Model>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::create_strategy(conn, request.name, request.description, request.status).await {
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
    pub trade_mode: String,
    pub status: i32,
    pub config: Option<serde_json::Value>,
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
}

pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateStrategyRequest>,
) -> (StatusCode, Json<ApiResponse<strategy_config::Model>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    tracing::info!("更新策略请求: {:?}", request);
    match StrategyConfigMutation::update_strategy_by_id(
        conn,
        request.id, 
        request.name, 
        request.description, 
        request.trade_mode,
        request.status, 
        request.config,
        request.nodes, 
        request.edges
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


#[derive(Serialize, Deserialize)]
pub struct AddMt5AccountConfigRequest {
    pub account_name: String,
    pub exchange: String,
    pub login: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
}

// 新增mt5账户配置
pub async fn add_mt5_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<AddMt5AccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<Mt5AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    match Mt5AccountConfigMutation::insert_mt5_account_config(
        conn, 
        request.account_name, 
        Exchange::from_str(request.exchange.as_str()).expect("Invalid exchange"), 
        request.login, 
        request.password, 
        request.server, 
        request.terminal_path
    ).await {
        Ok(account_config) => {
            // 添加成功之后，发布账户配置已添加事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigAdded((account_config.get_account_id(), account_config.get_exchange())));
            event_publisher.publish(event).unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(account_config),
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


#[derive(Serialize, Deserialize)]
pub struct DeleteMt5AccountConfigRequest {
    pub id: i32,
}


pub async fn delete_mt5_account_config(
    State(star_river): State<StarRiver>,
    Query(request): Query<DeleteMt5AccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match Mt5AccountConfigMutation::delete_mt5_account_config(conn, request.id).await {
        Ok(_) => {
            // 删除成功之后，发布账户配置已删除事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigDeleted(request.id));
            event_publisher.publish(event).unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "删除成功".to_string(),
                data: None,
            })
        )},
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
pub struct UpdateMt5AccountConfigRequest {
    pub id: i32,
    pub account_name: String,
    pub exchange: String,
    pub login: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
    pub is_available: bool,
    pub sort_index: i32,
}


pub async fn update_mt5_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateMt5AccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<Mt5AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match Mt5AccountConfigMutation::update_mt5_account_config(
        conn, 
        request.id, 
        request.account_name, 
        Exchange::from_str(request.exchange.as_str()).expect("Invalid exchange"), 
        request.login, 
        request.password, 
        request.server, 
        request.terminal_path, 
        request.is_available, 
        request.sort_index
    ).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(AccountConfig::MetaTrader5(account_config.clone())));
            event_publisher.publish(event).unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(account_config),
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
pub struct UpdateMt5AccountConfigIsAvailableRequest {
    pub id: i32,
    pub is_available: bool,
}


pub async fn update_mt5_account_config_is_available(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateMt5AccountConfigIsAvailableRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    tracing::info!("更新mt5账户配置的is_available: {:?}", request);
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match Mt5AccountConfigMutation::update_mt5_account_config_is_available(conn, request.id, request.is_available).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(AccountConfig::MetaTrader5(account_config.clone())));
            event_publisher.publish(event).unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: None,
            })
        )},
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


