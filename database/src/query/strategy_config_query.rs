use sea_orm::*;
use crate::entities::{strategy_config, strategy_config::Entity as StrategyConfig};


pub struct StrategyConfigQuery;


impl StrategyConfigQuery {

    // 分页查询策略列表
    pub async fn get_strategy_list_in_page(
        db: &DbConn,
        page: u64,
        strategy_per_page: u64
    ) -> Result<(Vec<strategy_config::Model>, u64), DbErr> {
        let paginator = StrategyConfig::find()
        // 只查询未删除的策略
        .filter(strategy_config::Column::IsDeleted.eq(false))
        .order_by_desc(strategy_config::Column::CreatedTime)
        .paginate(db, strategy_per_page);
        
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    // 获取所有未删除的策略
    pub async fn get_all_strategy(
        db: &DbConn,
    ) -> Result<Vec<strategy_config::Model>, DbErr> {
        let strategies = StrategyConfig::find().filter(strategy_config::Column::IsDeleted.eq(false)).all(db).await?;
        Ok(strategies)
    }

    pub async fn get_strategy_by_id(
        db: &DbConn,
        id: i32
    ) -> Result<Option<strategy_config::Model>, DbErr> {
        let strategy_config = StrategyConfig::find_by_id(id).one(db).await?;
        Ok(strategy_config)
    }
}

