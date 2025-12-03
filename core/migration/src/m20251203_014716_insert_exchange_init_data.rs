use chrono::Utc;
use entity::account_config::{ActiveModel as AccountConfigActiveModel, Entity as AccountConfigEntity};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{entity::*, query::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        AccountConfigActiveModel {
            id: NotSet,
            account_name: Set("Binance".to_string()),
            exchange: Set("binance".to_string()),
            is_available: Set(true),
            is_delete: Set(false),
            sort_index: Set(0),
            account_config: Set(serde_json::json!({})),
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();
        AccountConfigEntity::delete_many()
            .filter(::entity::account_config::Column::AccountName.eq("Binance".to_string()))
            .filter(::entity::account_config::Column::Exchange.eq("binance".to_string()))
            .exec(db)
            .await?;

        Ok(())
    }
}
