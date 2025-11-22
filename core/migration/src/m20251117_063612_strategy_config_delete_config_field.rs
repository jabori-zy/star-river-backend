use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite 不支持直接删除列，需要采用重建表的方式

        // 1. 创建新的临时表，不包含 config 字段
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

        // 2. 将数据从原表迁移到新表，不包含 config 字段
        let sql = r#"
            INSERT INTO strategy_config_new (
                id, name, description, status, is_deleted, trade_mode,
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            )
            SELECT
                id, name, description, status, is_deleted, trade_mode,
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            FROM strategy_config
        "#;
        manager.get_connection().execute_unprepared(sql).await?;

        // 3. 删除原表
        manager.drop_table(Table::drop().table(StrategyConfig::Table).to_owned()).await?;

        // 4. 将新表重命名为原表名
        manager
            .rename_table(Table::rename().table(StrategyConfigNew::Table, StrategyConfig::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：重新添加 config 字段

        // 1. 创建新的临时表，包含 config 字段
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
                    .col(ColumnDef::new(StrategyConfigOld::Status).string().not_null().default("stopped"))
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

        // 2. 将数据从当前表迁移到临时表，config 字段设置为 NULL
        let sql = r#"
            INSERT INTO strategy_config_old (
                id, name, description, status, is_deleted, trade_mode, config,
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            )
            SELECT
                id, name, description, status, is_deleted, trade_mode, NULL as config,
                nodes, edges, live_chart_config, backtest_chart_config, created_time, updated_time
            FROM strategy_config
        "#;
        manager.get_connection().execute_unprepared(sql).await?;

        // 3. 删除当前表
        manager.drop_table(Table::drop().table(StrategyConfig::Table).to_owned()).await?;

        // 4. 将临时表重命名为原表名
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
