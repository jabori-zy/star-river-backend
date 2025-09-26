use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SystemConfig::Table)
                    .add_column(ColumnDef::new(SystemConfig::Timezone).string().not_null().default("Asia/Shanghai"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SystemConfig::Table)
                    .drop_column(SystemConfig::Timezone)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    Table,
    Timezone,
}
