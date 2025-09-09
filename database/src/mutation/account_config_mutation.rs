use ::entity::account_config;
use chrono::Utc;
use sea_orm::*;
use types::account::AccountConfig;
use types::market::Exchange;

pub struct AccountConfigMutation;

impl AccountConfigMutation {
    pub async fn insert_account_config(
        db: &DbConn,
        account_name: String,
        exchange: Exchange,
        account_config: serde_json::Value,
    ) -> Result<AccountConfig, DbErr> {
        // 获取最大sort_index
        let max_sort_index = account_config::Entity::find()
            .order_by(account_config::Column::SortIndex, Order::Desc)
            .one(db)
            .await?;
        // 如果max_sort_index为None，则sort_index为0
        let sort_index = max_sort_index.map_or(0, |config| config.sort_index) + 1;
        let account_config_model = account_config::ActiveModel {
            id: NotSet,
            account_name: Set(account_name),
            exchange: Set(exchange.to_string()),
            is_available: Set(true),
            is_delete: Set(false),
            sort_index: Set(sort_index),
            account_config: Set(account_config),
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
        }
        .insert(db)
        .await?;
        Ok(account_config_model.into())
    }

    pub async fn update_account_config(
        db: &DbConn,
        id: i32,
        account_name: String,
        account_config: serde_json::Value,
        is_available: bool,
        sort_index: i32,
    ) -> Result<AccountConfig, DbErr> {
        // 获取mt5账户配置
        let account_config_active_model: account_config::ActiveModel =
            account_config::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
                .map(Into::into)?;

        let account_config_model = account_config::ActiveModel {
            id: account_config_active_model.id,
            account_name: Set(account_name),
            account_config: Set(account_config),
            is_available: Set(is_available),
            is_delete: Set(false),
            sort_index: Set(sort_index),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await
        .unwrap();

        Ok(account_config_model.into())
    }

    pub async fn delete_account_config(db: &DbConn, id: i32) -> Result<(), DbErr> {
        let account_config_model: account_config::ActiveModel =
            account_config::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
                .map(Into::into)?;

        account_config::ActiveModel {
            id: account_config_model.id,
            is_delete: Set(true), // 设置为删除状态
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(())
    }

    // 更新账户配置的is_available
    pub async fn update_account_config_is_available(
        db: &DbConn,
        id: i32,
        is_available: bool,
    ) -> Result<AccountConfig, DbErr> {
        let account_config_active_model: account_config::ActiveModel =
            account_config::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
                .map(Into::into)?;

        let account_config_model = account_config::ActiveModel {
            id: account_config_active_model.id,
            is_available: Set(is_available),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(account_config_model.into())
    }
}
