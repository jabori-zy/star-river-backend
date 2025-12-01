use snafu::{OptionExt, ResultExt};
use strategy_core::node::context_trait::NodeInfoExt;
use tokio::sync::oneshot;
use virtual_trading::{
    command::{CloseAllPositionsCmdPayload, CloseAllPositionsCommand, ClosePositionCmdPayload, ClosePositionCommand, VtsResponse},
    error::{CommandSendFailedSnafu, ResponseRecvFailedSnafu},
};

use super::PositionNodeContext;
use crate::{
    node::node_error::{PositionNodeError, position_node_error::SymbolNotConfiguredSnafu},
    node_catalog::position_node::position_node_types::{PositionOperation, PositionOperationConfig},
};

impl PositionNodeContext {
    pub async fn handle_position_operation(&self, config_id: i32) -> Result<(), PositionNodeError> {
        let config = self.node_config.find_position_operation_config(config_id)?;

        let op = &config.position_operation;
        match op {
            PositionOperation::ClosePosition => {
                self.close_position(config).await?;
            }
            PositionOperation::CloseAllPositions => {
                self.close_all_positions(config).await?;
            }
            _ => return Ok(()),
        }

        Ok(())
    }

    async fn close_position(&self, config: &PositionOperationConfig) -> Result<(), PositionNodeError> {
        let symbol = config
            .symbol
            .clone()
            .context(SymbolNotConfiguredSnafu {
                node_name: self.node_name(),
                op: config.position_operation.to_string(),
            })?
            .clone();

        let exchange = self.node_config.selected_account.exchange.clone();

        let (tx, rx) = oneshot::channel();
        let payload = ClosePositionCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            self.node_name().clone(),
            symbol,
            exchange,
            config.config_id,
        );
        let close_position_cmd = ClosePositionCommand::new(tx, payload);
        self.vts_command_sender
            .send(close_position_cmd.into())
            .await
            .context(CommandSendFailedSnafu {})?;
        let response = rx.await.context(ResponseRecvFailedSnafu {})?;
        match response {
            VtsResponse::Success { .. } => return Ok(()),
            VtsResponse::Fail { error, .. } => return Err(error.into()),
        }
    }

    async fn close_all_positions(&self, config: &PositionOperationConfig) -> Result<(), PositionNodeError> {
        let (tx, rx) = oneshot::channel();
        let payload = CloseAllPositionsCmdPayload::new(self.node_id().clone(), self.node_name().clone(), config.config_id);
        let close_all_positions_cmd = CloseAllPositionsCommand::new(tx, payload);
        self.vts_command_sender
            .send(close_all_positions_cmd.into())
            .await
            .context(CommandSendFailedSnafu {})?;
        let response = rx.await.context(ResponseRecvFailedSnafu {})?;
        match response {
            VtsResponse::Success { .. } => return Ok(()),
            VtsResponse::Fail { error, .. } => return Err(error.into()),
        }
    }
}
