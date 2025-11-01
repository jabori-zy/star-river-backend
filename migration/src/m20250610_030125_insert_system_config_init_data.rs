use ::entity::system_config::Entity as SystemConfig;
use chrono::Utc;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{entity::*, query::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 插入系统配置初始数据
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

        // 删除插入的初始数据
        SystemConfig::delete_many()
            .filter(::entity::system_config::Column::Localization.eq("en-US".to_string()))
            .exec(db)
            .await?;

        Ok(())
    }
}
