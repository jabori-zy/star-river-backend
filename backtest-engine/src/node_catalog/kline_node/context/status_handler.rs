// third-party
use event_center::{CmdRespRecvFailedSnafu, EventCenterSingleton};
use event_center_core::communication::response::Response;
use key::{KeyTrait, KlineKey, error::TimeRangeNotSetSnafu};
use snafu::{IntoError, OptionExt, ResultExt};
use star_river_core::{custom_type::AccountId, exchange::Exchange, kline::Kline};
use star_river_event::communication::{
    ExchangeEngineCommand, GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand, RegisterExchangeCmdPayload,
    RegisterExchangeCommand,
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategySnafu},
    node::context_trait::{NodeCommunicationExt, NodeInfoExt},
};
use tokio::sync::oneshot;
use tracing::instrument;

// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::{
    node::node_error::kline_node_error::{LoadKlineFromExchangeFailedSnafu, RegisterExchangeFailedSnafu},
    strategy::strategy_command::{AppendKlineDataCmdPayload, AppendKlineDataCommand, InitKlineDataCmdPayload, InitKlineDataCommand},
};

impl KlineNodeContext {
    // 从交易所获取k线历史(仅获取最小interval的k线)
    #[instrument(target = "backtest::kline::binance", skip(self))]
    pub async fn load_kline_history_from_exchange(&self) -> Result<(), KlineNodeError> {
        let account_id = self.node_config.exchange_mode()?.selected_account.account_id;

        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();

        let time_range = self.node_config.exchange_mode()?.time_range.clone();

        match exchange {
            Exchange::Metatrader5(_) => self.get_mt5_kline_history(account_id, &time_range).await?,
            Exchange::Binance => self.get_binance_kline_history(account_id, &time_range).await?,
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
            kline_key.exchange(),
            kline_key.symbol(),
            kline_key.interval(),
            kline_key.time_range().context(TimeRangeNotSetSnafu {
                exchange: kline_key.exchange().to_string(),
                symbol: kline_key.symbol().to_string(),
                interval: kline_key.interval().to_string(),
            })?,
        );
        let cmd: MarketEngineCommand = GetKlineHistoryCommand::new(node_id.clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await?;

        let response = resp_rx.await.context(CmdRespRecvFailedSnafu {})?;
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.kline_history.clone());
            }
            Response::Fail { error, .. } => {
                return Err(LoadKlineFromExchangeFailedSnafu {
                    exchange: kline_key.exchange().to_string(),
                }
                .into_error(error));
            }
        }
    }

    pub(super) async fn init_strategy_kline_data(&self, symbol_key: &KlineKey, kline_history: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = InitKlineDataCmdPayload::new(symbol_key.clone(), kline_history.clone());
        let init_kline_data_command = InitKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(init_kline_data_command.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { .. } => Ok(()),
            StrategyResponse::Fail { error, .. } => Err(StrategySnafu {
                node_name: self.node_name().clone(),
            }
            .into_error(error)
            .into()),
        }
    }

    pub(super) async fn append_kline_data(&self, symbol_key: &KlineKey, kline_series: &Vec<Kline>) -> Result<(), KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = AppendKlineDataCmdPayload::new(symbol_key.clone(), kline_series.clone());
        let append_kline_data_command = AppendKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(append_kline_data_command.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }

    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&self) -> Result<(), KlineNodeError> {
        let account_id = self.node_config.exchange_mode()?.selected_account.account_id;
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

        tracing::info!("[{}] start to register exchange [{}]", node_name, exchange);

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = RegisterExchangeCmdPayload::new(account_id, exchange);
        let cmd: ExchangeEngineCommand = RegisterExchangeCommand::new(node_id, resp_tx, payload).into();

        EventCenterSingleton::send_command(cmd.into()).await?;

        // 等待响应
        let response = resp_rx.await.context(CmdRespRecvFailedSnafu {})?;
        match response {
            Response::Success { .. } => {
                return Ok(());
            }
            Response::Fail { error, .. } => {
                return Err(RegisterExchangeFailedSnafu {
                    node_name: node_name.clone(),
                }
                .into_error(error));
            }
        }
    }
}
