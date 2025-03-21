use sea_orm::*;
use crate::entities::{strategy_info, strategy_info::Entity as StrategyInfo};


pub struct StrategyInfoQuery;


impl StrategyInfoQuery {

    // 分页查询策略列表
    pub async fn get_strategy_list_in_page(
        db: &DbConn,
        page: u64,
        strategy_per_page: u64
    ) -> Result<(Vec<strategy_info::Model>, u64), DbErr> {
        let paginator = StrategyInfo::find()
        // 只查询未删除的策略
        .filter(strategy_info::Column::IsDeleted.eq(0))
        .order_by_desc(strategy_info::Column::CreatedTime)
        .paginate(db, strategy_per_page);
        
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    // 获取所有未删除的策略
    pub async fn get_all_strategy(
        db: &DbConn,
    ) -> Result<Vec<strategy_info::Model>, DbErr> {
        let strategies = StrategyInfo::find().filter(strategy_info::Column::IsDeleted.eq(0)).all(db).await?;
        Ok(strategies)
    }

    pub async fn get_strategy_by_id(
        db: &DbConn,
        id: i32
    ) -> Result<Option<strategy_info::Model>, DbErr> {
        let strategy_info = StrategyInfo::find_by_id(id).one(db).await?;
        Ok(strategy_info)
    }
}

