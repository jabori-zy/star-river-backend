use ::entity::{strategy_config, strategy_config::Entity as StrategyConfigEntity};
use chrono::Utc;
use sea_orm::*;
use star_river_core::strategy::StrategyConfig;

pub struct StrategyConfigMutation;

impl StrategyConfigMutation {
    pub async fn create_strategy(
        db: &DbConn,
        strategy_name: String,
        strategy_description: String,
    ) -> Result<StrategyConfig, DbErr> {
        let strategy_config_model = strategy_config::ActiveModel {
            id: NotSet,
            name: Set(strategy_name),
            description: Set(strategy_description),
            status: Set("stopped".to_string()),
            is_deleted: Set(false),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
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
        strategy_config: Option<JsonValue>,
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
            config: Set(strategy_config),
            nodes: Set(nodes),
            edges: Set(edges),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }

    pub async fn delete_strategy(db: &DbConn, strategy_id: i32) -> Result<(), DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        strategy_config::ActiveModel {
            id: strategy.id,
            is_deleted: Set(true), // 设置为删除状态
            updated_time: Set(Utc::now()),
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
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.backtest_chart_config.unwrap_or(JsonValue::Null))
    }

    pub async fn update_strategy_status(
        db: &DbConn,
        strategy_id: i32,
        strategy_status: String,
    ) -> Result<StrategyConfig, DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfigEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            status: Set(strategy_status),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }
}
