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
                    .col(ColumnDef::new(StrategyConfig::Status).integer().not_null().default(0))
                    .col(ColumnDef::new(StrategyConfig::IsDeleted).boolean().not_null().default(false))
                    .col(ColumnDef::new(StrategyConfig::TradeMode).string().not_null().default("backtest"))
                    .col(ColumnDef::new(StrategyConfig::Config).json())
                    .col(ColumnDef::new(StrategyConfig::Nodes).json())
                    .col(ColumnDef::new(StrategyConfig::Edges).json())
                    .col(ColumnDef::new(StrategyConfig::LiveChartConfig).json())
                    .col(ColumnDef::new(StrategyConfig::BacktestChartConfig).json())
                    .col(
                        ColumnDef::new(StrategyConfig::CreatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(StrategyConfig::UpdatedTime)
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
    Name,                //策略名称
    Description,         //策略描述
    Status,              //策略状态 1=开启 0=关闭
    IsDeleted,           //是否删除 0=正常 1=删除
    TradeMode,           //交易模式 实盘，模拟，回测
    Config,              //策略配置
    Nodes,               //节点
    Edges,               //边
    LiveChartConfig,     //实盘图表配置
    BacktestChartConfig, //回测图表配置
    CreatedTime,         //创建时间
    UpdatedTime,         //更新时间
}
