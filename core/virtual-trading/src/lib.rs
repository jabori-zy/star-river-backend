pub mod command;
pub mod context;
pub mod error;
pub mod event;

pub mod types;
pub(crate) mod utils;
pub mod vts_trait;

#[cfg(test)]
mod test;

use std::{fmt::Debug, sync::Arc};

pub use context::VtsContext;
use tokio::sync::RwLock;
use vts_trait::{VTSEventListener, VtsCtxAccessor};

#[derive(Debug, Clone)]
pub struct Vts<C: Debug + Send + Sync + 'static> {
    pub context: Arc<RwLock<C>>,
}

impl<C> Vts<C>
where
    C: Debug + Send + Sync + 'static,
{
    pub fn new(context: C) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

// VTSAccessor trait implementation

impl<C> vts_trait::VtsCtxAccessor for Vts<C>
where
    C: Debug + Send + Sync + 'static,
{
    type Context = C;

    fn context(&self) -> &Arc<RwLock<C>> {
        &self.context
    }
}

impl<C> Vts<C>
where
    C: Debug + Send + Sync + 'static + VTSEventListener,
{
    pub async fn start(&self) {
        tracing::info!("ready to start virtual trading system...");
        // 1. start listen kline node events
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                // 1.1 listen kline node events
                ctx.listen_kline_node_events().await;
                // 1.2 listen vts command
                ctx.listen_vts_command().await;
            })
        })
        .await;
        tracing::info!("virtual trading system started");
    }
}
