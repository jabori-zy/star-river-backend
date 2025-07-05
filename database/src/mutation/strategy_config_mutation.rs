use sea_orm::*;
use ::entity::{strategy_config, strategy_config::Entity as StrategyConfig};
use chrono::Utc;
use types::strategy::Strategy;


pub struct StrategyConfigMutation;


impl StrategyConfigMutation {
    pub async fn create_strategy(
        db: &DbConn,
        strategy_name: String,
        strategy_description: String,
        strategy_status: i32,
    ) -> Result<Strategy, DbErr> {
        let strategy_config_model = strategy_config::ActiveModel {
            id: NotSet,
            name: Set(strategy_name),
            description: Set(strategy_description),
            status: Set(strategy_status),
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
        strategy_status: i32,
        strategy_config: Option<JsonValue>,
        nodes: Option<JsonValue>,
        edges: Option<JsonValue>,
        live_chart_config: Option<JsonValue>,
        backtest_chart_config: Option<JsonValue>,
    ) -> Result<Strategy, DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfig::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_config_model = strategy_config::ActiveModel {
            id: strategy.id,
            name: Set(strategy_name),
            description: Set(strategy_description),
            trade_mode: Set(strategy_trade_mode),
            status: Set(strategy_status),
            config: Set(strategy_config),
            nodes: Set(nodes),
            edges: Set(edges),
            live_chart_config: Set(live_chart_config),
            backtest_chart_config: Set(backtest_chart_config),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(strategy_config_model.into())
    }


    pub async fn delete_strategy(
        db: &DbConn,
        strategy_id: i32,
    ) -> Result<(), DbErr> {
        let strategy: strategy_config::ActiveModel = StrategyConfig::find_by_id(strategy_id)
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
        let strategy: strategy_config::ActiveModel = StrategyConfig::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
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

        
}

