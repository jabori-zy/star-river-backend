use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000009_create_strategy_statistics_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountInfo::Table)
                    .col(
                        ColumnDef::new(AccountInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AccountInfo::AccountId).integer().not_null()) // Account config ID
                    .col(ColumnDef::new(AccountInfo::Info).json()) // Account info
                    .col(ColumnDef::new(AccountInfo::CreateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(ColumnDef::new(AccountInfo::UpdateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountInfo::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum StrategyStatistics { // Strategy statistics table
    Table,
    Id,
    StrategyId,
    PositionNumber, // Position quantity
    CreateTime,
    UpdateTime,
}