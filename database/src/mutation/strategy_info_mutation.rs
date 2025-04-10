use sea_orm::*;
use crate::entities::{strategy_info, strategy_info::Entity as StrategyInfo};
use chrono::Utc;


pub struct StrategyInfoMutation;


impl StrategyInfoMutation {
    pub async fn create_strategy(
        db: &DbConn,
        strategy_name: String,
        strategy_description: String,
        strategy_status: i32,
    ) -> Result<strategy_info::Model, DbErr> {
        strategy_info::ActiveModel {
            id: NotSet,
            name: Set(strategy_name),
            description: Set(strategy_description),
            status: Set(strategy_status),
            is_deleted: Set(0),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    pub async fn update_strategy_by_id(
        db: &DbConn,
        strategy_id: i32,
        strategy_name: String,
        strategy_description: String,
        strategy_status: i32,
        nodes: Option<JsonValue>,
        edges: Option<JsonValue>,
    ) -> Result<strategy_info::Model, DbErr> {
        let strategy: strategy_info::ActiveModel = StrategyInfo::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        strategy_info::ActiveModel {
            id: strategy.id,
            name: Set(strategy_name),
            description: Set(strategy_description),
            status: Set(strategy_status),
            nodes: Set(nodes),
            edges: Set(edges),
            is_deleted: Set(0),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await
    }


    pub async fn delete_strategy(
        db: &DbConn,
        strategy_id: i32,
    ) -> Result<(), DbErr> {
        let strategy: strategy_info::ActiveModel = StrategyInfo::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        strategy_info::ActiveModel {
            id: strategy.id,
            is_deleted: Set(1), // 设置为删除状态
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())



    }

        
}

