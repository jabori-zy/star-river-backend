use star_river_core::account::mt5_account::Mt5AccountConfig as TypeMt5AccountConfig;
use sea_orm::*;
use crate::entities::mt5_account_config;
use chrono::Utc;
use star_river_core::market::Exchange;


pub struct Mt5AccountConfigMutation;


impl Mt5AccountConfigMutation {

    pub async fn insert_mt5_account_config(
        db: &DbConn,
        account_name: String,
        exchange: Exchange,
        login: i64,
        password: String,
        server: String,
        terminal_path: String,
    ) -> Result<TypeMt5AccountConfig, DbErr> {
        // Check if same account_id and terminal_path exist
        let account_config_model= mt5_account_config::Entity::find()
            .filter(mt5_account_config::Column::Login.eq(login))
            .filter(mt5_account_config::Column::TerminalPath.eq(terminal_path.clone()))
            .one(db)
            .await?;

        // If same account_id and terminal_path exist, set is_deleted to false
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

            // Get maximum sort_index
            let max_sort_index = mt5_account_config::Entity::find()
                .order_by(mt5_account_config::Column::SortIndex, Order::Desc)
                .one(db)
                .await?;
            // If max_sort_index is None, set sort_index to 0
            let sort_index = max_sort_index.map_or(0, |config| config.sort_index) + 1;
            let account_config_model = mt5_account_config::ActiveModel {
                id: NotSet,
                account_name: Set(account_name),
                exchange: Set(exchange.to_string()),
                is_available: Set(true),
                login: Set(login),
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
        login: i64,
        password: String,
        server: String,
        terminal_path: String,
        is_available: bool,
        sort_index: i32,
    ) -> Result<TypeMt5AccountConfig, DbErr> {
        // Get MT5 account configuration
        let account_config_active_model: mt5_account_config::ActiveModel = mt5_account_config::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
            .map(Into::into)?;

        let account_config_model = mt5_account_config::ActiveModel {
            id: account_config_active_model.id,
            account_name: Set(account_name),
            exchange: Set(exchange.to_string()),
            login: Set(login),
            password: Set(password),
            server: Set(server),
            terminal_path: Set(terminal_path),
            is_available: Set(is_available),
            is_deleted: Set(false),
            sort_index: Set(sort_index),
            updated_time: Set(Utc::now()),
            ..Default::default()
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
            is_deleted: Set(true), // Set to deleted state
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(())
    }

    // Update MT5 account configuration is_available field
    pub async fn update_mt5_account_config_is_available(
        db: &DbConn,
        id: i32,
        is_available: bool,
    ) -> Result<TypeMt5AccountConfig, DbErr> {
        let account_config_active_model: mt5_account_config::ActiveModel = mt5_account_config::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find mt5 account config.".to_owned()))
            .map(Into::into)?;

        let account_config_model = mt5_account_config::ActiveModel {
            id: account_config_active_model.id,
            is_available: Set(is_available),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(account_config_model.into())
    }

}

