use super::{AccountId, Engine, ExchangeEngineContext, MarketEngineContext, MarketEngineError, Symbol};

impl MarketEngineContext {
    pub async fn get_symbol_list(&self, account_id: AccountId) -> Result<Vec<Symbol>, MarketEngineError> {
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await?;
        let symbol_list = exchange.get_symbol_list().await?;
        Ok(symbol_list)
    }

    pub async fn get_symbol(&self, account_id: AccountId, symbol: String) -> Result<Symbol, MarketEngineError> {
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await?;
        let symbol = exchange.get_symbol(symbol).await?;
        Ok(symbol)
    }
}
