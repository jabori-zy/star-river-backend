use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemConfig::Table)
                    .if_not_exists()
                    .col(pk_auto(SystemConfig::Id))
                    .col(string(SystemConfig::Localization))
                    .col(timestamp(SystemConfig::CreatedTime).default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(timestamp(SystemConfig::UpdatedTime).default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SystemConfig::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    Table,
    Id,
    Localization,
    CreatedTime,
    UpdatedTime,
}
