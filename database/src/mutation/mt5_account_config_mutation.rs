use types::account::mt5_account::Mt5AccountConfig as TypeMt5AccountConfig;
use sea_orm::*;
use crate::entities::mt5_account_config;
use chrono::Utc;
use types::market::Exchange;


pub struct Mt5AccountConfigMutation;


impl Mt5AccountConfigMutation {

    pub async fn insert_mt5_account_config(
        db: &DbConn,
        account_name: String,
        exchange: Exchange,
        account_id: i64,
        password: String,
        server: String,
        terminal_path: String,
    ) -> Result<TypeMt5AccountConfig, DbErr> {
        // 检查是否存在相同的account_id和terminal_path
        let account_config_model= mt5_account_config::Entity::find()
            .filter(mt5_account_config::Column::AccountId.eq(account_id))
            .filter(mt5_account_config::Column::TerminalPath.eq(terminal_path.clone()))
            .one(db)
            .await?;

        // 如果存在相同的account_id和terminal_path，则将is_deleted设置为true
        if account_config_model.is_some() {
            let mt5_account_config_model: mt5_account_config::ActiveModel = account_config_model.unwrap().into();

           let mt5_account_config = mt5_account_config::ActiveModel {
                id: mt5_account_config_model.id,
                is_deleted: Set(false),
                created_time: Set(Utc::now()),
                updated_time: Set(Utc::now()),
                ..Default::default()
            }
            .update(db)
            .await?;
            Ok(mt5_account_config.into())
        } else {

            // 获取最大sort_index
            let max_sort_index = mt5_account_config::Entity::find()
                .order_by(mt5_account_config::Column::SortIndex, Order::Desc)
                .one(db)
                .await?;
            // 如果max_sort_index为None，则sort_index为0
            let sort_index = max_sort_index.map_or(0, |config| config.sort_index) + 1;
            let account_config_model = mt5_account_config::ActiveModel {
                id: NotSet,
                account_name: Set(account_name),
                exchange: Set(exchange.to_string()),
                is_available: Set(true),
                account_id: Set(account_id),
                password: Set(password),
                server: Set(server),
                terminal_path: Set(terminal_path),
                is_deleted: Set(false),
                sort_index: Set(sort_index),
                created_time: Set(Utc::now()),
                updated_time: Set(Utc::now()),
            }.insert(db).await?;
            Ok(account_config_model.into())
        }

        
    }

    pub async fn update_mt5_account_config(
        db: &DbConn,
        id: i32,
        account_name: String,
        exchange: Exchange,
        account_id: i64,
        password: String,
        server: String,
        terminal_path: String,
        is_available: bool,
        sort_index: i32,
    ) -> Result<TypeMt5AccountConfig, DbErr> {
        // 获取mt5账户配置
        let account_config_model: mt5_account_config::ActiveModel = mt5_account_config::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
            .map(Into::into)?;

        let account_config_model = mt5_account_config::ActiveModel {
            id: account_config_model.id,
            account_name: Set(account_name),
            exchange: Set(exchange.to_string()),
            account_id: Set(account_id),
            password: Set(password),
            server: Set(server),
            terminal_path: Set(terminal_path),
            is_available: Set(is_available),
            is_deleted: Set(false),
            sort_index: Set(sort_index),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
        }.update(db).await.unwrap();

        Ok(account_config_model.into())
    }


    pub async fn delete_mt5_account_config(
        db: &DbConn,
        id: i32,
    ) -> Result<(), DbErr> {
        let account_config_model: mt5_account_config::ActiveModel = mt5_account_config::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
            .map(Into::into)?;

        mt5_account_config::ActiveModel {
            id: account_config_model.id,
            is_deleted: Set(true), // 设置为删除状态
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(())
    }

}

