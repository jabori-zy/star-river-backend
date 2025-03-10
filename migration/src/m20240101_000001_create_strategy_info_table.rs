use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000003_create_strategy_info_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StrategyInfo::Table)
                    .col(
                        ColumnDef::new(StrategyInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StrategyInfo::Name).string().not_null())
                    .col(ColumnDef::new(StrategyInfo::Description).string().not_null())
                    .col(ColumnDef::new(StrategyInfo::Status).integer().not_null().default(0))
                    .col(ColumnDef::new(StrategyInfo::IsDeleted).integer().not_null().default(0))
                    .col(ColumnDef::new(StrategyInfo::Nodes).json())
                    .col(ColumnDef::new(StrategyInfo::Edges).json())
                    .col(ColumnDef::new(StrategyInfo::CreatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(ColumnDef::new(StrategyInfo::UpdatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
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
        manager
            .drop_table(Table::drop().table(StrategyInfo::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum StrategyInfo {
    Table,
    Id,
    Name,//策略名称
    Description,//策略描述
    Status,//策略状态 1=开启 0=关闭
    IsDeleted,//是否删除 0=正常 1=删除
    Nodes,//节点
    Edges,//边
    CreatedTime,//创建时间
    UpdatedTime,//更新时间
}