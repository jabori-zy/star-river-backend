use ::entity::system_config::Entity as SystemConfig;
use chrono::Utc;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{entity::*, query::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Insert system configuration initial data
        ::entity::system_config::ActiveModel {
            localization: Set("en-US".to_string()),
            timezone: Set(String::from("Asia/Shanghai")),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Delete inserted initial data
        SystemConfig::delete_many()
            .filter(::entity::system_config::Column::Localization.eq("en-US".to_string()))
            .exec(db)
            .await?;

        Ok(())
    }
}
