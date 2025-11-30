use strategy_core::{error::node_error::OutputHandleNotFoundSnafu, node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt}};

use super::PositionNodeContext;
use crate::node::node_error::PositionNodeError;
use futures::{stream, TryStreamExt};

impl PositionNodeContext {
    pub(super) async fn independent_position_op_send_trigger_event(
        &self,
        config_id: i32,
        context: Option<String>,
    ) -> Result<(), PositionNodeError> {
        let all_output_handles = self.output_handles();
        tracing::debug!("send trigger event to position output handles: {:#?}", all_output_handles);
        
        // Collect matching handles
        let handles: Vec<_> = self
            .output_handles()
            .values()
            .filter(|handle| handle.config_id() == config_id)
            .collect();
        
        // Check if any handle found
        if handles.is_empty() {
            return Err(OutputHandleNotFoundSnafu {
                node_name: self.node_name().clone(),
                handle_id: None,
                config_id: Some(config_id),
            }.build().into());
        }
        
        // Process with stream - same pattern as variable_node
        stream::iter(handles.iter().map(|handle| Ok::<_, PositionNodeError>(handle)))
            .try_for_each_concurrent(None, |handle| async {
                self.send_trigger_event(
                    handle.output_handle_id(),
                    Some(config_id),
                    context.clone(),
                    Some(self.strategy_time()),
                )
                .await?;
                Ok(())
            })
            .await?;

        Ok(())
    }
}

