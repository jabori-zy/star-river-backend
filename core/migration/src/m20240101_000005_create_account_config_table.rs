use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000005_create_account_config_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountConfig::Table)
                    .col(
                        ColumnDef::new(AccountConfig::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AccountConfig::AccountName).string().not_null())
                    .col(ColumnDef::new(AccountConfig::Exchange).string().not_null())
                    .col(ColumnDef::new(AccountConfig::IsAvailable).boolean().not_null())
                    .col(ColumnDef::new(AccountConfig::IsDelete).boolean().not_null())
                    .col(ColumnDef::new(AccountConfig::SortIndex).integer().not_null())
                    .col(ColumnDef::new(AccountConfig::AccountConfig).json().not_null())
                    .col(
                        ColumnDef::new(AccountConfig::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(AccountConfig::UpdateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    // .foreign_key(
                    //     ForeignKey::create()
                    //         .name("fk-strategy_info-bakery_id")
                    //         .from(StrategyInfo::Table, StrategyInfo::BakeryId)
                    //         .to(Bakery::Table, Bakery::Id),
                    // )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(AccountConfig::Table).to_owned()).await
    }
}

// For ease of access
#[derive(Iden)]
pub enum AccountConfig {
    Table,
    Id,            // Primary key
    AccountName,   // Account name
    Exchange,      // Exchange
    IsAvailable,   // Is available
    IsDelete,      // Is deleted
    SortIndex,     // Sort index
    AccountConfig, // Account config
    CreateTime,    // Created time
    UpdateTime,    // Updated time
}
