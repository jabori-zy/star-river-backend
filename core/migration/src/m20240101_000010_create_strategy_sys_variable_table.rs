use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000010_create_strategy_sys_variable_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StrategySysVariable::Table)
                    .col(
                        ColumnDef::new(StrategySysVariable::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StrategySysVariable::StrategyId).integer().not_null()) // Strategy ID
                    .col(ColumnDef::new(StrategySysVariable::PositionNumber).integer().not_null().default(0)) // Position quantity
                    .col(
                        ColumnDef::new(StrategySysVariable::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(StrategySysVariable::UpdateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(StrategySysVariable::Table).to_owned()).await
    }
}

// For ease of access
#[derive(Iden)]
pub enum StrategySysVariable {
    Table,
    Id,
    StrategyId,
    PositionNumber,
    CreateTime,
    UpdateTime,
}
