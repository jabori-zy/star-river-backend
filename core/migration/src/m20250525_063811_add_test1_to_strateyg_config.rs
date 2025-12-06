use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add test1 field to strategy_config table
        manager
            .alter_table(
                Table::alter()
                    .table(StrategyConfig::Table)
                    .add_column(
                        ColumnDef::new(StrategyConfig::Test1)
                            .string()
                            .null() // Set to nullable to avoid affecting existing data
                            .default("") // Optional: set default value
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove test1 field on rollback
        manager
            .alter_table(
                Table::alter()
                    .table(StrategyConfig::Table)
                    .drop_column(StrategyConfig::Test1)
                    .to_owned(),
            )
            .await
    }
}

// Define table and field identifiers
#[derive(Iden)]
enum StrategyConfig {
    Table,
    Test1, // Newly added test field
}
