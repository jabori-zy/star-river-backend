// third-party
use event_center::EventCenterSingleton;
use event_center_core::communication::response::Response;
use key::{KeyTrait, KlineKey};
use snafu::IntoError;
use star_river_core::{custom_type::AccountId, exchange::Exchange, kline::Kline};
use star_river_event::communication::{
    ExchangeEngineCommand, GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand, RegisterExchangeCmdPayload,
    RegisterExchangeCommand, RegisterExchangeRespPayload,
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    node::context_trait::{NodeCommunicationExt, NodeIdentityExt},
};
use tokio::sync::oneshot;
use tracing::instrument;

// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::{
    node::node_error::kline_node_error::{AppendKlineDataFailedSnafu, InitKlineDataFailedSnafu, LoadKlineFromExchangeFailedSnafu},
    strategy::strategy_command::{AppendKlineDataCmdPayload, AppendKlineDataCommand, InitKlineDataCmdPayload, InitKlineDataCommand},
};

impl KlineNodeContext {
    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(target = "backtest::kline::binance", skip(self))]
    pub async fn load_kline_history_from_exchange(&self) -> Result<(), KlineNodeError> {
        tracing::info!("[{}] start to load backtest kline data from exchange", self.node_name());

        let account_id = self
            .node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();

        let exchange = self
            .node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();

        let time_range = self.node_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        match exchange {
            Exchange::Metatrader5(_) => self.get_mt5_kline_history(account_id.clone(), &time_range).await?,
            Exchange::Binance => self.get_binance_kline_history(account_id.clone(), &time_range).await?,
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }

    // request kline history from market engine
    pub(super) async fn request_kline_history(&self, account_id: AccountId, kline_key: &KlineKey) -> Result<Vec<Kline>, KlineNodeError> {
        let node_id = self.node_id().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineHistoryCmdPayload::new(
            self.strategy_id().clone(),
            node_id.clone(),
            account_id.clone(),
            kline_key.get_exchange(),
            kline_key.get_symbol(),
            kline_key.get_interval(),
            kline_key.get_time_range().unwrap(),
        );
        let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.kline_history.clone());
            }
            Response::Fail { error, .. } => {
                return Err(LoadKlineFromExchangeFailedSnafu {
                    exchange: kline_key.get_exchange().to_string(),
                }
                .into_error(error));
            }
        }
    }

    pub(super) async fn init_strategy_kline_data(&self, symbol_key: &KlineKey, kline_history: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = InitKlineDataCmdPayload::new(symbol_key.clone(), kline_history.clone());
        let init_kline_data_command = InitKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.strategy_command_sender().send(init_kline_data_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(InitKlineDataFailedSnafu {}.into_error(error));
            }
        }
    }

    pub(super) async fn append_kline_data(&self, symbol_key: &KlineKey, kline_series: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = AppendKlineDataCmdPayload::new(symbol_key.clone(), kline_series.clone());
        let append_kline_data_command = AppendKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.strategy_command_sender().send(append_kline_data_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(AppendKlineDataFailedSnafu {}.into_error(error));
            }
        }
    }

    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&self) -> Result<Response<RegisterExchangeRespPayload>, String> {
        let account_id = self
            .node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self
            .node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

        tracing::info!("[{}] start to register exchange [{}]", node_name, exchange);

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = RegisterExchangeCmdPayload::new(account_id, exchange);
        let cmd: ExchangeEngineCommand = RegisterExchangeCommand::new(node_id, resp_tx, payload).into();

        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }
}
