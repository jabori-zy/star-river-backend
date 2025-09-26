use super::{
    KlineNodeContext,
    GetKlineHistoryCmdPayload,
    GetKlineHistoryCommand,
    MarketEngineCommand,
    InitKlineDataCmdPayload,
    InitKlineDataCommand,
    EventCenterSingleton,
    BacktestNodeContextTrait,
    KeyTrait,
    Response,
    RegisterExchangeCmdPayload,
    ExchangeEngineCommand,
    EngineResponse,
    RegisterExchangeRespPayload,
    RegisterExchangeCommand
};
use tokio::sync::oneshot;
use tracing::instrument;


impl KlineNodeContext {
    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!(
            "[{}] start to load backtest kline data from exchange",
            self.base_context.node_name
        );

        let mut is_all_success = true;

        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();

        // 遍历每一个symbol，从交易所获取k线历史
        for (symbol_key, _) in self.selected_symbol_keys.iter() {
            // 如果key不在最小周期交易对列表中，则跳过
            if !self.min_interval_symbols.contains(&symbol_key) {
                tracing::warn!(
                    "[{}] symbol: {}-{}, is not min interval, skip",
                    self.get_node_name(),
                    symbol_key.get_symbol(),
                    symbol_key.get_interval()
                );
                continue;
            }
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetKlineHistoryCmdPayload::new(
                strategy_id,
                node_id.clone(),
                account_id.clone(),
                symbol_key.get_exchange(),
                symbol_key.get_symbol(),
                symbol_key.get_interval(),
                symbol_key.get_time_range().unwrap(),
            );
            let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(),resp_tx,Some(payload)).into();
            EventCenterSingleton::send_command(cmd.into())
                .await
                .unwrap();

            let response = resp_rx.await.unwrap();
            if response.is_success() {
                let kline_history = response.kline_history.clone();
                tracing::debug!(
                    "[{}] get kline history from exchange success, symbol: {}-{}, kline history length: {:#?}",
                    self.get_node_name(),
                    symbol_key.get_symbol(),
                    symbol_key.get_interval(),
                    kline_history.len()
                );

                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = InitKlineDataCmdPayload::new(
                    symbol_key.clone(),
                    kline_history,
                );
                
                let init_kline_data_command= InitKlineDataCommand::new(
                    self.get_node_id().clone(),
                    resp_tx,
                    Some(payload),
                );

                self.get_strategy_command_sender()
                    .send(init_kline_data_command.into())
                    .await
                    .unwrap();

                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    continue;
                }
                
            }

            else {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }


    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<EngineResponse<RegisterExchangeRespPayload>, String> {
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();

        tracing::info!("[{}] start to register exchange [{}]", node_name, exchange);

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = RegisterExchangeCmdPayload::new(
            account_id, 
            exchange
        );
        let cmd: ExchangeEngineCommand = RegisterExchangeCommand::new(node_id, resp_tx, Some(payload)).into();

        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }
}