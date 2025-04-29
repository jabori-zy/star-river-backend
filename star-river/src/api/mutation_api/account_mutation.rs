


use axum::extract::{Json, Query, State};
use types::account::AccountConfig;
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;
use database::mutation::account_config_mutation::AccountConfigMutation;
use types::market::Exchange;
use std::str::FromStr;
use types::account::mt5_account::Mt5AccountConfig;
use event_center::Event;
use event_center::account_event::AccountEvent;
use types::account::ExchangeAccountConfig;


#[derive(Serialize, Deserialize)]
pub struct AddAccountConfigRequest {
    pub account_name: String,
    pub exchange: String,
    pub account_config: serde_json::Value,
}

// 新增账户配置
pub async fn add_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<AddAccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    match AccountConfigMutation::insert_account_config(
        conn, 
        request.account_name, 
        Exchange::from_str(request.exchange.as_str()).expect("Invalid exchange"), 
        request.account_config
    ).await {
        Ok(account_config) => {
            // 添加成功之后，发布账户配置已添加事件
            tracing::info!("添加账户配置成功: {:?}", account_config);
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigAdded(account_config.id));
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
pub struct DeleteAccountConfigRequest {
    pub id: i32,
}


pub async fn delete_account_config(
    State(star_river): State<StarRiver>,
    Query(request): Query<DeleteAccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::delete_account_config(conn, request.id).await {
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
pub struct UpdateAccountConfigRequest {
    pub id: i32,
    pub account_name: String,
    pub account_config: serde_json::Value,
    pub is_available: bool,
    pub sort_index: i32,
}


pub async fn update_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateAccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::update_account_config(
        conn, 
        request.id, 
        request.account_name, 
        request.account_config, 
        request.is_available, 
        request.sort_index
    ).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
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
pub struct UpdateIsAvailableRequest {
    pub id: i32,
    pub is_available: bool,
}


pub async fn update_account_config_is_available(
    State(star_river): State<StarRiver>,
    Json(request): Json<UpdateIsAvailableRequest>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    tracing::info!("更新mt5账户配置的is_available: {:?}", request);
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::update_account_config_is_available(conn, request.id, request.is_available).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
            event_publisher.publish(event).unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(account_config),
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