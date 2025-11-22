use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 strategy_config 表添加 test1 字段
        manager
            .alter_table(
                Table::alter()
                    .table(StrategyConfig::Table)
                    .add_column(
                        ColumnDef::new(StrategyConfig::Test1)
                            .string()
                            .null() // 设为可空，这样不会影响现有数据
                            .default("") // 可选：设置默认值
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚时删除 test1 字段
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

// 定义表和字段标识符
#[derive(Iden)]
enum StrategyConfig {
    Table,
    Test1, // 新增的测试字段
}
