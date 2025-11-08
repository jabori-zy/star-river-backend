use engine_core::EngineContextAccessor;
use star_river_core::custom_type::AccountId;
use star_river_core::instrument::Symbol;
use crate::error::MarketEngineError;
use super::MarketEngineContext;

impl MarketEngineContext {
    /// 获取指定账户的交易品种列表
    pub async fn get_symbol_list(&self, account_id: AccountId) -> Result<Vec<Symbol>, MarketEngineError> {
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let symbol_list = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let exchange = ctx.get_exchange_instance(&account_id).await?;
                    let symbol_list = exchange.symbol_list().await?;
                    Ok::<Vec<Symbol>, MarketEngineError>(symbol_list)
                })
            })
            .await?;

        Ok(symbol_list)
    }

    /// 获取指定账户的单个交易品种信息
    pub async fn get_symbol(&self, account_id: AccountId, symbol: String) -> Result<Symbol, MarketEngineError> {
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let symbol = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let exchange = ctx.get_exchange_instance(&account_id).await?;
                    let symbol = exchange.symbol(symbol).await?;
                    Ok::<Symbol, MarketEngineError>(symbol)
                })
            })
            .await?;

        Ok(symbol)
    }
}
