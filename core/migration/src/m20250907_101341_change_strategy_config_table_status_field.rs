use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite does not support altering column types directly, need to rebuild table

        // 1. Create new temporary table with status field as string type
        manager
            .create_table(
                Table::create()
                    .table(StrategyConfigNew::Table)
                    .col(
                        ColumnDef::new(StrategyConfigNew::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StrategyConfigNew::Name).string().not_null())
                    .col(ColumnDef::new(StrategyConfigNew::Description).string().not_null())
                    .col(ColumnDef::new(StrategyConfigNew::Status).string().not_null().default("stopped"))
                    .col(ColumnDef::new(StrategyConfigNew::IsDeleted).boolean().not_null().default(false))
                    .col(ColumnDef::new(StrategyConfigNew::TradeMode).string().not_null().default("backtest"))
                    .col(ColumnDef::new(StrategyConfigNew::Config).json())
                    .col(ColumnDef::new(StrategyConfigNew::Nodes).json())
                    .col(ColumnDef::new(StrategyConfigNew::Edges).json())
                    .col(ColumnDef::new(StrategyConfigNew::LiveChartConfig).json())
                    .col(ColumnDef::new(StrategyConfigNew::BacktestChartConfig).json())
                    .col(
                        ColumnDef::new(StrategyConfigNew::CreatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(StrategyConfigNew::UpdatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Migrate data from original table to new table, convert integer status to string
        let sql = r#"
            INSERT INTO strategy_config_new (
                id, name, description, status, is_deleted, trade_mode, config, 
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            )
            SELECT 
                id, name, description, 
                CASE 
                    WHEN status = 1 THEN 'running'
                    WHEN status = 0 THEN 'stopped'
                    ELSE 'stopped'
                END as status,
                is_deleted, trade_mode, config, 
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            FROM strategy_config
        "#;
        manager.get_connection().execute_unprepared(sql).await?;

        // 3. Drop original table
        manager.drop_table(Table::drop().table(StrategyConfig::Table).to_owned()).await?;

        // 4. Rename new table to original table name
        manager
            .rename_table(Table::rename().table(StrategyConfigNew::Table, StrategyConfig::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rollback: change status field from string back to integer type

        // 1. Create new temporary table with status field as integer type
        manager
            .create_table(
                Table::create()
                    .table(StrategyConfigOld::Table)
                    .col(
                        ColumnDef::new(StrategyConfigOld::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StrategyConfigOld::Name).string().not_null())
                    .col(ColumnDef::new(StrategyConfigOld::Description).string().not_null())
                    .col(ColumnDef::new(StrategyConfigOld::Status).integer().not_null().default(0))
                    .col(ColumnDef::new(StrategyConfigOld::IsDeleted).boolean().not_null().default(false))
                    .col(ColumnDef::new(StrategyConfigOld::TradeMode).string().not_null().default("backtest"))
                    .col(ColumnDef::new(StrategyConfigOld::Config).json())
                    .col(ColumnDef::new(StrategyConfigOld::Nodes).json())
                    .col(ColumnDef::new(StrategyConfigOld::Edges).json())
                    .col(ColumnDef::new(StrategyConfigOld::LiveChartConfig).json())
                    .col(ColumnDef::new(StrategyConfigOld::BacktestChartConfig).json())
                    .col(
                        ColumnDef::new(StrategyConfigOld::CreatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(StrategyConfigOld::UpdatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Migrate data from current table to temporary table, convert string status to integer
        let sql = r#"
            INSERT INTO strategy_config_old (
                id, name, description, status, is_deleted, trade_mode, config, 
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            )
            SELECT 
                id, name, description, 
                CASE 
                    WHEN status = 'running' THEN 1
                    WHEN status = 'stopped' THEN 0
                    ELSE 0
                END as status,
                is_deleted, trade_mode, config, 
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            FROM strategy_config
        "#;
        manager.get_connection().execute_unprepared(sql).await?;

        // 3. Drop current table
        manager.drop_table(Table::drop().table(StrategyConfig::Table).to_owned()).await?;

        // 4. Rename temporary table to original table name
        manager
            .rename_table(Table::rename().table(StrategyConfigOld::Table, StrategyConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum StrategyConfig {
    Table,
    Id,
    Name,
    Description,
    Status,
    IsDeleted,
    TradeMode,
    Config,
    Nodes,
    Edges,
    LiveChartConfig,
    BacktestChartConfig,
    CreatedTime,
    UpdatedTime,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum StrategyConfigNew {
    Table,
    Id,
    Name,
    Description,
    Status,
    IsDeleted,
    TradeMode,
    Config,
    Nodes,
    Edges,
    LiveChartConfig,
    BacktestChartConfig,
    CreatedTime,
    UpdatedTime,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum StrategyConfigOld {
    Table,
    Id,
    Name,
    Description,
    Status,
    IsDeleted,
    TradeMode,
    Config,
    Nodes,
    Edges,
    LiveChartConfig,
    BacktestChartConfig,
    CreatedTime,
    UpdatedTime,
}
