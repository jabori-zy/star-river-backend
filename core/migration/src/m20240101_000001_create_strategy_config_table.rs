use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000001_create_strategy_config_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StrategyConfig::Table)
                    .col(
                        ColumnDef::new(StrategyConfig::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StrategyConfig::Name).string().not_null())
                    .col(ColumnDef::new(StrategyConfig::Description).string().not_null())
                    .col(ColumnDef::new(StrategyConfig::Status).string().not_null().default("stopped"))
                    .col(ColumnDef::new(StrategyConfig::IsDeleted).boolean().not_null().default(false))
                    .col(ColumnDef::new(StrategyConfig::TradeMode).string().not_null().default("backtest"))
                    .col(ColumnDef::new(StrategyConfig::Nodes).json())
                    .col(ColumnDef::new(StrategyConfig::Edges).json())
                    .col(ColumnDef::new(StrategyConfig::LiveChartConfig).json())
                    .col(ColumnDef::new(StrategyConfig::BacktestChartConfig).json())
                    .col(
                        ColumnDef::new(StrategyConfig::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(StrategyConfig::UpdateTime)
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
        manager.drop_table(Table::drop().table(StrategyConfig::Table).to_owned()).await
    }
}

// For ease of access
#[derive(Iden)]
pub enum StrategyConfig {
    Table,
    Id,
    Name,                // Strategy name
    Description,         // Strategy description
    Status,              // Strategy status: 1=running 0=stopped
    IsDeleted,           // Is deleted: 0=normal 1=deleted
    TradeMode,           // Trading mode: live, paper, backtest
    Nodes,               // Nodes
    Edges,               // Edges
    LiveChartConfig,     // Live chart config
    BacktestChartConfig, // Backtest chart config
    CreateTime,          // Created time
    UpdateTime,          // Updated time
}
