use ::entity::{strategy_config, strategy_config::Entity as StrategyConfigEntity};
use chrono::Utc;
use sea_orm::*;
use strategy_core::strategy::StrategyConfig;

use crate::error::DatabaseError;

pub struct StrategyConfigMutation;

impl StrategyConfigMutation {
    pub async fn create_strategy(
        db: &DbConn,
        strategy_name: String,
        strategy_description: String,
    ) -> Result<StrategyConfig, DatabaseError> {
        let strategy_config_model = strategy_config::ActiveModel {
            id: NotSet,
            name: Set(strategy_name),
            description: Set(strategy_description),
            status: Set("Stopped".to_string()),
            is_deleted: Set(false),
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn update_strategy_by_id(
        db: &DbConn,
        strategy_id: i32,
        strategy_name: String,
        strategy_description: String,
        strategy_trade_mode: String,
        nodes: Option<JsonValue>,
        edges: Option<JsonValue>,
    ) -> Result<StrategyConfig, DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            name: Set(strategy_name),
            description: Set(strategy_description),
            trade_mode: Set(strategy_trade_mode),
            nodes: Set(nodes),
            edges: Set(edges),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn delete_strategy(db: &DbConn, strategy_id: i32) -> Result<(), DatabaseError> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        strategy_config::ActiveModel {
            id: strategy.id,
            is_deleted: Set(true), // 设置为删除状态
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())
    }

    // 更新回测图表配置
    pub async fn update_backtest_chart_config(
        db: &DbConn,
        strategy_id: i32,
        backtest_chart_config: Option<JsonValue>,
    ) -> Result<JsonValue, DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            backtest_chart_config: Set(backtest_chart_config),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.backtest_chart_config.unwrap_or(JsonValue::Null))
    }

    pub async fn update_strategy_status(db: &DbConn, strategy_id: i32, strategy_status: String) -> Result<StrategyConfig, DatabaseError> {
        tracing::info!(
            "update strategy status: strategy_id: {}, strategy_status: {}",
            strategy_id,
            strategy_status
        );
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            status: Set(strategy_status),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn update_strategy_name(
        db: &DbConn,
        strategy_id: i32,
        name: String,
        description: String,
    ) -> Result<StrategyConfig, DatabaseError> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Cannot find strategy {strategy_id}.")))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            name: Set(name),
            description: Set(description),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn update_strategy_trade_mode(db: &DbConn, strategy_id: i32, trade_mode: String) -> Result<StrategyConfig, DatabaseError> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Cannot find strategy {strategy_id}.")))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            trade_mode: Set(trade_mode),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn update_strategy_workflow(
        db: &DbConn,
        strategy_id: i32,
        nodes: Option<JsonValue>,
        edges: Option<JsonValue>,
    ) -> Result<StrategyConfig, DatabaseError> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Cannot find strategy {strategy_id}.")))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            nodes: Set(nodes),
            edges: Set(edges),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }
}
